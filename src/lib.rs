use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

pub mod codegen;

#[derive(Debug, Default)]
pub struct BeansContext {
  beans: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
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
}

pub trait Bean {
  fn create(ctx: &BeansContext) -> Self;
}

#[cfg(test)]
mod tests {
  use super::*;
  use beany_codegen::Bean;

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
}
