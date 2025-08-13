use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

pub type BoxError = Box<dyn Error + Send + Sync>;
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait StreamConn: Send + Sync {
    fn read<'a>(&'a mut self) -> BoxFuture<'a, Result<Vec<u8>, BoxError>>;
    fn write<'a>(&'a mut self, bytes: Vec<u8>) -> BoxFuture<'a, Result<(), BoxError>>;
    fn remote_addr(&self) -> SocketAddr;
}

pub trait Listener: Send + Sync {
    fn run<'a>(&'a self) -> BoxFuture<'a, Result<(), BoxError>>;
}

