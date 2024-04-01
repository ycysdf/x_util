use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::{ready, FutureExt};
use pin_project::pin_project;
use tokio::time::error::Elapsed;
use tokio::time::Timeout;

pub trait FutureTimeoutExt: Future {
    fn timeout(self, duration: Duration) -> Timeout<Self>
    where
        Self: Sized,
    {
        tokio::time::timeout(duration, self)
    }

    fn timeout_optional(self, duration: Option<Duration>) -> TimeoutOptionalFuture<Self>
    where
        Self: Sized,
    {
        match duration {
            None => TimeoutOptionalFuture::Origin(self),
            Some(duration) => TimeoutOptionalFuture::Timeout(tokio::time::timeout(duration, self)),
        }
    }
}

#[pin_project(project = TimeoutOptionalFutureProj)]
pub enum TimeoutOptionalFuture<T>
where
    T: Future,
{
    Origin(#[pin] T),
    Timeout(#[pin] Timeout<T>),
}

impl<T> Future for TimeoutOptionalFuture<T>
where
    T: Future,
{
    type Output = Result<T::Output, Elapsed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            TimeoutOptionalFutureProj::Origin(n) => Poll::Ready(Ok(ready!(n.poll(cx)))),
            TimeoutOptionalFutureProj::Timeout(n) => n.poll(cx),
        }
    }
}

impl<T> FutureTimeoutExt for T where T: Future {}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use futures_util::FutureExt;
    use tokio::time::error::Elapsed;

    use crate::timeout::FutureTimeoutExt;

    #[tokio::test]
    async fn timeout_test() -> color_eyre::Result<()> {
        let pin = async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        .boxed();
        pin.timeout(Duration::from_secs(1)).await.unwrap_err();
        Ok(())
    }
    #[tokio::test]
    async fn timeout_test2() -> color_eyre::Result<()> {
        let pin = async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        .boxed();
        assert_eq!(pin.timeout(Duration::from_secs(2)).await, Ok(()));
        Ok(())
    }
}
