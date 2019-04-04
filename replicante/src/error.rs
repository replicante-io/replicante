use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;
use iron::IronError;

use replicante_util_iron::into_ironerror;


/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

impl From<::failure::Error> for Error {
    fn from(error: ::failure::Error) -> Error {
        ErrorKind::Legacy(error).into()
    }
}


/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "could not initialise client interface for {}", _0)]
    ClientInit(&'static str),

    #[fail(display = "could not load configuration")]
    ConfigLoad,

    #[fail(display = "could not coordinate with other processes")]
    Coordination,

    #[fail(display = "could not deserialize {} into {}", _0, _1)]
    Deserialize(&'static str, &'static str),

    #[fail(display = "could not initialise {} interface", _0)]
    InterfaceInit(&'static str),

    #[fail(display = "could not find model {} with ID {}", _0, _1)]
    ModelNotFound(&'static str, String),

    #[fail(display = "could not query {} from the primary store", _0)]
    PrimaryStoreQuery(&'static str),

    #[fail(display = "could not persist {} model to primary store", _0)]
    PrimaryStorePersist(&'static str),

    #[fail(display = "could not spawn new thread for '{}'", _0)]
    SpawnThread(&'static str),

    #[fail(display = "could not register task worker for queue '{}'", _0)]
    TaskWorkerRegistration(String),

    // TODO: drop once all uses are removed.
    #[fail(display = "{}", _0)]
    #[deprecated(since = "0.2.0", note = "move to specific ErrorKinds")]
    Legacy(#[cause] ::failure::Error),
}


/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;


// IronError compatibility code.
impl From<Error> for IronError {
    fn from(error: Error) -> Self {
        into_ironerror(error)
    }
}


#[cfg(test)]
mod tests {
    use failure::Fail;
    use failure::err_msg;

    use iron::IronResult;
    use iron::Headers;
    use iron::Response;
    use iron::Request;
    use iron::headers::ContentType;

    use iron_test::request;
    use iron_test::response;

    use super::Error;
    use super::ErrorKind;

    fn failing(_: &mut Request) -> IronResult<Response> {
        let error: Error = err_msg("test")
            .context(ErrorKind::Legacy(err_msg("chained")))
            .context(ErrorKind::Legacy(err_msg("failures")))
            .into();
        Err(error.into())
    }

    #[test]
    fn error_conversion() {
        let response = request::get("http://host:16016/", Headers::new(), &failing);
        let response = match response {
            Err(error) => error.response,
            Ok(_) => panic!("Request should fail")
        };

        let content_type = response.headers.get::<ContentType>().unwrap().clone();
        assert_eq!(content_type, ContentType::json());

        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, r#"{"error":"failures","layers":["failures","chained","test"],"trace":null}"#);
    }
}
