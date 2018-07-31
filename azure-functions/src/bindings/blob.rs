use blob::Contents;
use rpc::protocol;

/// Represents a Azure Storage blob input or output binding.
///
/// Blobs can be created for any type that implements `Into<Contents>`.
///
/// # Examples
///
/// Creating a blob from a string:
///
/// ```rust
/// use azure_functions::bindings::Blob;
///
/// let blob: Blob = "hello world!".into();
/// assert_eq!(blob.contents().as_str().unwrap(), "hello world!");
/// ```
///
/// Creating a blob from a JSON value (see the [json! macro](https://docs.serde.rs/serde_json/macro.json.html) from the `serde_json` crate):
///
/// ```rust
/// # #[macro_use] extern crate serde_json;
/// # extern crate azure_functions;
/// use azure_functions::bindings::Blob;
///
/// let blob: Blob = json!({ "hello": "world!" }).into();
///
/// assert_eq!(blob.contents().as_str().unwrap(), r#"{"hello":"world!"}"#);
/// ```
///
/// Creating a blob from a sequence of bytes:
///
/// ```rust
/// use azure_functions::bindings::Blob;
///
/// let blob: Blob = [1, 2, 3][..].into();
///
/// assert_eq!(
///     blob.contents().as_bytes(),
///     [1, 2, 3]
/// );
/// ```
#[derive(Debug)]
pub struct Blob(protocol::TypedData);

impl Blob {
    /// Gets the contents of the blob.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::Blob;
    ///
    /// let blob: Blob = "hello world!".into();
    /// assert_eq!(blob.contents().as_str().unwrap(), "hello world!");
    /// ```
    pub fn contents(&self) -> Contents {
        (&self.0).into()
    }
}

impl<T> From<T> for Blob
where
    T: Into<Contents<'_>>,
{
    fn from(data: T) -> Self {
        let contents: Contents = data.into();
        Blob(contents.into())
    }
}

impl Into<protocol::TypedData> for Blob {
    fn into(self) -> protocol::TypedData {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_string_content() {
        const CONTENT: &'static str = "test content";

        let blob: Blob = CONTENT.into();
        assert_eq!(blob.contents().as_str().unwrap(), CONTENT);

        let data: protocol::TypedData = blob.into();
        assert_eq!(data.get_string(), CONTENT);
    }

    #[test]
    fn it_has_json_contents() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            message: String,
        };

        const MESSAGE: &'static str = "test";

        let data = Data {
            message: MESSAGE.to_string(),
        };

        let blob: Blob = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(
            blob.contents().from_json::<Data>().unwrap().message,
            MESSAGE
        );

        let data: protocol::TypedData = blob.into();
        assert_eq!(data.get_json(), r#"{"message":"test"}"#);
    }

    #[test]
    fn it_has_a_bytes_content() {
        const CONTENT: &'static [u8] = &[1, 2, 3];

        let blob: Blob = CONTENT.into();
        assert_eq!(blob.contents().as_bytes(), CONTENT);

        let data: protocol::TypedData = blob.into();
        assert_eq!(data.get_bytes(), CONTENT);
    }
}
