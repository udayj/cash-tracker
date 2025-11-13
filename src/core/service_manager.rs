use tokio::sync::mpsc;
use tokio::task::JoinSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use super::Error;
use super::service::{Service, ServiceWithReceiver};
use tracing::error;

pub struct ServiceManager<C> {
    context: C,
    services: JoinSet<()>,
}

impl<C> ServiceManager<C>
where
    C: 'static + Clone + Send,
{
    pub fn new(context: C) -> Self {
        Self {
            context,
            services: JoinSet::new(),
        }
    }

    pub fn spawn<T: Service<Context = C>>(&mut self, error_channel: mpsc::Sender<String>) {
        let context = self.context.clone();
        self.services.spawn(async move {
            loop {
                let service = T::new(context.clone(), error_channel.clone()).await;
                if let Err(_) = service.run().await {
                    continue;
                }
            }
        });
    }

    pub fn spawn_with_error_receiver<T: ServiceWithReceiver<Context = C>>(
        &mut self,
        receiver: Arc<Mutex<mpsc::Receiver<String>>>,
    ) {
        let context = self.context.clone();
        self.services.spawn(async move {
            loop {
                let service = T::new(context.clone(), Some(receiver.clone())).await;
                if let Err(e) = service.run().await {
                    error!(service = std::any::type_name::<T>(), error = %e, "Service error");
                    break;
                }
            }
        });
    }

    pub async fn wait(&mut self) -> Result<(), Error> {
        if self.services.join_next().await.is_some() {
            return Err(Error::new("Internal Service Error"));
        }
        Ok(())
    }
}
