use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use parking_lot::RwLock;

pub mod codegen;

#[derive(Debug, Default)]
pub struct BeansContext {
  beans: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
  async_beans: tokio::sync::RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl BeansContext {
  pub fn get<T>(&self) -> Arc<T>
  where
    T: Bean + Send + Sync + 'static,
  {
    let read = self.beans.read();

    match read.get(&TypeId::of::<T>()) {
      None => {
        drop(read);
        let bean = Arc::new(T::create(self));
        let mut write = self.beans.write();
        write.insert(TypeId::of::<T>(), bean.clone());
        bean
      }
      Some(bean) => bean.clone().downcast().expect("bean type mismatch"),
    }
  }

  pub async fn get_async<T>(&self) -> Arc<T>
  where
    T: AsyncBean + Send + Sync + 'static,
  {
    let read = self.async_beans.read().await;

    match read.get(&TypeId::of::<T>()) {
      None => {
        drop(read);
        let bean = Arc::new(T::create(self).await);
        let mut write = self.async_beans.write().await;
        write.insert(TypeId::of::<T>(), bean.clone());
        bean
      }
      Some(bean) => bean.clone().downcast().expect("bean type mismatch"),
    }
  }
}

pub trait Bean {
  fn create(ctx: &BeansContext) -> Self;
}

pub trait AsyncBean {
  fn create(ctx: &BeansContext) -> impl Future<Output = Self> + Send;
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate as beany;
  use beany_codegen::{AsyncBean, Bean};

  #[derive(Bean, Clone)]
  struct Messager {
    message: Arc<Message>,
  }

  #[derive(Bean)]
  struct Message;

  #[derive(Bean, Clone)]
  struct TestService {
    messager: Messager,
  }

  #[test]
  fn test_di() {
    let ctx = BeansContext::default();

    let test_service = ctx.get::<TestService>();
  }

  #[derive(AsyncBean, Clone)]
  struct AsyncMessager {
    message: Arc<AsyncMessage>,
  }

  #[derive(AsyncBean)]
  struct AsyncMessage;

  #[derive(AsyncBean, Clone)]
  struct AsyncTestService {
    messager: AsyncMessager,
  }

  #[tokio::test]
  async fn test_async_di() {
    let ctx = BeansContext::default();

    let test_service = ctx.get_async::<AsyncTestService>().await;
  }
}
