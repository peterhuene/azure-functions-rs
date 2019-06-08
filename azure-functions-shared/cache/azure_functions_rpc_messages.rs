#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullableString {
    #[prost(oneof = "nullable_string::String", tags = "1")]
    pub string: ::std::option::Option<nullable_string::String>,
}
pub mod nullable_string {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum String {
        #[prost(string, tag = "1")]
        Value(std::string::String),
    }
}
/// Light-weight representation of a .NET System.Security.Claims.ClaimsIdentity object.
/// This is the same serialization as found in EasyAuth, and needs to be kept in sync with
/// its ClaimsIdentitySlim definition, as seen in the WebJobs extension:
/// https://github.com/Azure/azure-webjobs-sdk-extensions/blob/dev/src/WebJobs.Extensions.Http/ClaimsIdentitySlim.cs
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcClaimsIdentity {
    #[prost(message, optional, tag = "1")]
    pub authentication_type: ::std::option::Option<NullableString>,
    #[prost(message, optional, tag = "2")]
    pub name_claim_type: ::std::option::Option<NullableString>,
    #[prost(message, optional, tag = "3")]
    pub role_claim_type: ::std::option::Option<NullableString>,
    #[prost(message, repeated, tag = "4")]
    pub claims: ::std::vec::Vec<RpcClaim>,
}
/// Light-weight representation of a .NET System.Security.Claims.Claim object.
/// This is the same serialization as found in EasyAuth, and needs to be kept in sync with
/// its ClaimSlim definition, as seen in the WebJobs extension:
/// https://github.com/Azure/azure-webjobs-sdk-extensions/blob/dev/src/WebJobs.Extensions.Http/ClaimSlim.cs
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcClaim {
    #[prost(string, tag = "1")]
    pub value: std::string::String,
    #[prost(string, tag = "2")]
    pub r#type: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamingMessage {
    /// Used to identify message between host and worker
    #[prost(string, tag = "1")]
    pub request_id: std::string::String,
    /// Payload of the message
    #[prost(
        oneof = "streaming_message::Content",
        tags = "20, 17, 16, 15, 14, 12, 13, 6, 7, 8, 9, 4, 5, 21, 2, 25, 26"
    )]
    pub content: ::std::option::Option<streaming_message::Content>,
}
pub mod streaming_message {
    /// Payload of the message
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Content {
        /// Worker initiates stream
        #[prost(message, tag = "20")]
        StartStream(super::StartStream),
        /// Host sends capabilities/init data to worker
        #[prost(message, tag = "17")]
        WorkerInitRequest(super::WorkerInitRequest),
        /// Worker responds after initializing with its capabilities & status
        #[prost(message, tag = "16")]
        WorkerInitResponse(super::WorkerInitResponse),
        /// Worker periodically sends empty heartbeat message to host
        #[prost(message, tag = "15")]
        WorkerHeartbeat(super::WorkerHeartbeat),
        /// Host sends terminate message to worker.
        /// Worker terminates if it can, otherwise host terminates after a grace period
        #[prost(message, tag = "14")]
        WorkerTerminate(super::WorkerTerminate),
        /// Add any worker relevant status to response
        #[prost(message, tag = "12")]
        WorkerStatusRequest(super::WorkerStatusRequest),
        #[prost(message, tag = "13")]
        WorkerStatusResponse(super::WorkerStatusResponse),
        /// On file change event, host sends notification to worker
        #[prost(message, tag = "6")]
        FileChangeEventRequest(super::FileChangeEventRequest),
        /// Worker requests a desired action (restart worker, reload function)
        #[prost(message, tag = "7")]
        WorkerActionResponse(super::WorkerActionResponse),
        /// Host sends required metadata to worker to load function
        #[prost(message, tag = "8")]
        FunctionLoadRequest(super::FunctionLoadRequest),
        /// Worker responds after loading with the load result
        #[prost(message, tag = "9")]
        FunctionLoadResponse(super::FunctionLoadResponse),
        /// Host requests a given invocation
        #[prost(message, tag = "4")]
        InvocationRequest(super::InvocationRequest),
        /// Worker responds to a given invocation
        #[prost(message, tag = "5")]
        InvocationResponse(super::InvocationResponse),
        /// Host sends cancel message to attempt to cancel an invocation.
        /// If an invocation is cancelled, host will receive an invocation response with status cancelled.
        #[prost(message, tag = "21")]
        InvocationCancel(super::InvocationCancel),
        /// Worker logs a message back to the host
        #[prost(message, tag = "2")]
        RpcLog(super::RpcLog),
        #[prost(message, tag = "25")]
        FunctionEnvironmentReloadRequest(super::FunctionEnvironmentReloadRequest),
        #[prost(message, tag = "26")]
        FunctionEnvironmentReloadResponse(super::FunctionEnvironmentReloadResponse),
    }
}
// Process.Start required info
//   connection details
//   protocol type
//   protocol version

/// Worker sends the host information identifying itself
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartStream {
    /// id of the worker
    #[prost(string, tag = "2")]
    pub worker_id: std::string::String,
}
/// Host requests the worker to initialize itself
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkerInitRequest {
    /// version of the host sending init request
    #[prost(string, tag = "1")]
    pub host_version: std::string::String,
    /// A map of host supported features/capabilities
    #[prost(map = "string, string", tag = "2")]
    pub capabilities: ::std::collections::HashMap<std::string::String, std::string::String>,
    /// inform worker of supported categories and their levels
    /// i.e. Worker = Verbose, Function.MyFunc = None
    #[prost(map = "string, enumeration(rpc_log::Level)", tag = "3")]
    pub log_categories: ::std::collections::HashMap<std::string::String, i32>,
}
/// Worker responds with the result of initializing itself
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkerInitResponse {
    /// Version of worker
    #[prost(string, tag = "1")]
    pub worker_version: std::string::String,
    /// A map of worker supported features/capabilities
    #[prost(map = "string, string", tag = "2")]
    pub capabilities: ::std::collections::HashMap<std::string::String, std::string::String>,
    /// Status of the response
    #[prost(message, optional, tag = "3")]
    pub result: ::std::option::Option<StatusResult>,
}
/// Used by the host to determine success/failure/cancellation
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusResult {
    /// Status for the given result
    #[prost(enumeration = "status_result::Status", tag = "4")]
    pub status: i32,
    /// Specific message about the result
    #[prost(string, tag = "1")]
    pub result: std::string::String,
    /// Exception message (if exists) for the status
    #[prost(message, optional, tag = "2")]
    pub exception: ::std::option::Option<RpcException>,
    /// Captured logs or relevant details can use the logs property
    #[prost(message, repeated, tag = "3")]
    pub logs: ::std::vec::Vec<RpcLog>,
}
pub mod status_result {
    /// Indicates Failure/Success/Cancelled
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Status {
        Failure = 0,
        Success = 1,
        Cancelled = 2,
    }
}
// TODO: investigate grpc heartbeat - don't limit to grpc implemention

/// Message is empty by design - Will add more fields in future if needed
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkerHeartbeat {}
/// Warning before killing the process after grace_period
/// Worker self terminates ..no response on this
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkerTerminate {
    #[prost(message, optional, tag = "1")]
    pub grace_period: ::std::option::Option<::prost_types::Duration>,
}
/// Host notifies worker of file content change
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileChangeEventRequest {
    /// type for this event
    #[prost(enumeration = "file_change_event_request::Type", tag = "1")]
    pub r#type: i32,
    /// full file path for the file change notification
    #[prost(string, tag = "2")]
    pub full_path: std::string::String,
    /// Name of the function affected
    #[prost(string, tag = "3")]
    pub name: std::string::String,
}
pub mod file_change_event_request {
    /// Types of File change operations (See link for more info: https://msdn.microsoft.com/en-us/library/t6xf43e0(v=vs.110).aspx)
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unknown = 0,
        Created = 1,
        Deleted = 2,
        Changed = 4,
        Renamed = 8,
        All = 15,
    }
}
/// Indicates whether worker reloaded successfully or needs a restart
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkerActionResponse {
    /// action for this response
    #[prost(enumeration = "worker_action_response::Action", tag = "1")]
    pub action: i32,
    /// text reason for the response
    #[prost(string, tag = "2")]
    pub reason: std::string::String,
}
pub mod worker_action_response {
    /// indicates whether a restart is needed, or reload succesfully
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Action {
        Restart = 0,
        Reload = 1,
    }
}
/// NOT USED
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkerStatusRequest {}
/// NOT USED
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkerStatusResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionEnvironmentReloadRequest {
    /// Environment variables from the current process
    #[prost(map = "string, string", tag = "1")]
    pub environment_variables:
        ::std::collections::HashMap<std::string::String, std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionEnvironmentReloadResponse {
    /// Status of the response
    #[prost(message, optional, tag = "3")]
    pub result: ::std::option::Option<StatusResult>,
}
/// Host tells the worker to load a Function
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionLoadRequest {
    /// unique function identifier (avoid name collisions, facilitate reload case)
    #[prost(string, tag = "1")]
    pub function_id: std::string::String,
    /// Metadata for the request
    #[prost(message, optional, tag = "2")]
    pub metadata: ::std::option::Option<RpcFunctionMetadata>,
    /// A flag indicating if managed dependency is enabled or not
    #[prost(bool, tag = "3")]
    pub managed_dependency_enabled: bool,
}
/// Worker tells host result of reload
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionLoadResponse {
    /// unique function identifier
    #[prost(string, tag = "1")]
    pub function_id: std::string::String,
    /// Result of load operation
    ///
    /// TODO: return type expected?
    #[prost(message, optional, tag = "2")]
    pub result: ::std::option::Option<StatusResult>,
    /// Result of load operation
    #[prost(bool, tag = "3")]
    pub is_dependency_downloaded: bool,
}
/// Information on how a Function should be loaded and its bindings
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcFunctionMetadata {
    /// TODO: do we want the host's name - the language worker might do a better job of assignment than the host
    #[prost(string, tag = "4")]
    pub name: std::string::String,
    /// base directory for the Function
    #[prost(string, tag = "1")]
    pub directory: std::string::String,
    /// Script file specified
    #[prost(string, tag = "2")]
    pub script_file: std::string::String,
    /// Entry point specified
    #[prost(string, tag = "3")]
    pub entry_point: std::string::String,
    /// Bindings info
    #[prost(map = "string, message", tag = "6")]
    pub bindings: ::std::collections::HashMap<std::string::String, BindingInfo>,
}
/// Host requests worker to invoke a Function
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InvocationRequest {
    /// Unique id for each invocation
    #[prost(string, tag = "1")]
    pub invocation_id: std::string::String,
    /// Unique id for each Function
    #[prost(string, tag = "2")]
    pub function_id: std::string::String,
    /// Input bindings (include trigger)
    #[prost(message, repeated, tag = "3")]
    pub input_data: ::std::vec::Vec<ParameterBinding>,
    /// binding metadata from trigger
    #[prost(map = "string, message", tag = "4")]
    pub trigger_metadata: ::std::collections::HashMap<std::string::String, TypedData>,
}
/// Host requests worker to cancel invocation
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InvocationCancel {
    /// Unique id for invocation
    #[prost(string, tag = "2")]
    pub invocation_id: std::string::String,
    /// Time period before force shutdown
    ///
    /// could also use absolute time
    #[prost(message, optional, tag = "1")]
    pub grace_period: ::std::option::Option<::prost_types::Duration>,
}
/// Worker responds with status of Invocation
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InvocationResponse {
    /// Unique id for invocation
    #[prost(string, tag = "1")]
    pub invocation_id: std::string::String,
    /// Output binding data
    #[prost(message, repeated, tag = "2")]
    pub output_data: ::std::vec::Vec<ParameterBinding>,
    /// data returned from Function (for $return and triggers with return support)
    #[prost(message, optional, tag = "4")]
    pub return_value: ::std::option::Option<TypedData>,
    /// Status of the invocation (success/failure/canceled)
    #[prost(message, optional, tag = "3")]
    pub result: ::std::option::Option<StatusResult>,
}
/// Used to encapsulate data which could be a variety of types
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TypedData {
    #[prost(oneof = "typed_data::Data", tags = "1, 2, 3, 4, 5, 6, 7")]
    pub data: ::std::option::Option<typed_data::Data>,
}
pub mod typed_data {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        #[prost(string, tag = "1")]
        String(std::string::String),
        #[prost(string, tag = "2")]
        Json(std::string::String),
        #[prost(bytes, tag = "3")]
        Bytes(std::vec::Vec<u8>),
        #[prost(bytes, tag = "4")]
        Stream(std::vec::Vec<u8>),
        #[prost(message, tag = "5")]
        Http(Box<super::RpcHttp>),
        #[prost(sint64, tag = "6")]
        Int(i64),
        #[prost(double, tag = "7")]
        Double(f64),
    }
}
/// Used to describe a given binding on invocation
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParameterBinding {
    /// Name for the binding
    #[prost(string, tag = "1")]
    pub name: std::string::String,
    /// Data for the binding
    #[prost(message, optional, tag = "2")]
    pub data: ::std::option::Option<TypedData>,
}
/// Used to describe a given binding on load
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BindingInfo {
    /// Type of binding (e.g. HttpTrigger)
    #[prost(string, tag = "2")]
    pub r#type: std::string::String,
    /// Direction of the given binding
    #[prost(enumeration = "binding_info::Direction", tag = "3")]
    pub direction: i32,
    #[prost(enumeration = "binding_info::DataType", tag = "4")]
    pub data_type: i32,
}
pub mod binding_info {
    /// Indicates whether it is an input or output binding (or a fancy inout binding)
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Direction {
        In = 0,
        Out = 1,
        Inout = 2,
    }
    /// Indicates the type of the data for the binding
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum DataType {
        Undefined = 0,
        String = 1,
        Binary = 2,
        Stream = 3,
    }
}
/// Used to send logs back to the Host
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcLog {
    /// Unique id for invocation (if exists)
    #[prost(string, tag = "1")]
    pub invocation_id: std::string::String,
    /// TOD: This should be an enum
    /// Category for the log (startup, load, invocation, etc.)
    #[prost(string, tag = "2")]
    pub category: std::string::String,
    /// Level for the given log message
    #[prost(enumeration = "rpc_log::Level", tag = "3")]
    pub level: i32,
    /// Message for the given log
    #[prost(string, tag = "4")]
    pub message: std::string::String,
    /// Id for the even associated with this log (if exists)
    #[prost(string, tag = "5")]
    pub event_id: std::string::String,
    /// Exception (if exists)
    #[prost(message, optional, tag = "6")]
    pub exception: ::std::option::Option<RpcException>,
    /// json serialized property bag, or could use a type scheme like map<string, TypedData>
    #[prost(string, tag = "7")]
    pub properties: std::string::String,
}
pub mod rpc_log {
    /// Matching ILogger semantics
    /// https://github.com/aspnet/Logging/blob/9506ccc3f3491488fe88010ef8b9eb64594abf95/src/Microsoft.Extensions.Logging/Logger.cs
    /// Level for the Log
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Level {
        Trace = 0,
        Debug = 1,
        Information = 2,
        Warning = 3,
        Error = 4,
        Critical = 5,
        None = 6,
    }
}
/// Encapsulates an Exception
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcException {
    /// Source of the exception
    #[prost(string, tag = "3")]
    pub source: std::string::String,
    /// Stack trace for the exception
    #[prost(string, tag = "1")]
    pub stack_trace: std::string::String,
    /// Textual message describing hte exception
    #[prost(string, tag = "2")]
    pub message: std::string::String,
}
/// TODO - solidify this or remove it
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcHttp {
    #[prost(string, tag = "1")]
    pub method: std::string::String,
    #[prost(string, tag = "2")]
    pub url: std::string::String,
    #[prost(map = "string, string", tag = "3")]
    pub headers: ::std::collections::HashMap<std::string::String, std::string::String>,
    #[prost(message, optional, boxed, tag = "4")]
    pub body: ::std::option::Option<::std::boxed::Box<TypedData>>,
    #[prost(map = "string, string", tag = "10")]
    pub params: ::std::collections::HashMap<std::string::String, std::string::String>,
    #[prost(string, tag = "12")]
    pub status_code: std::string::String,
    #[prost(map = "string, string", tag = "15")]
    pub query: ::std::collections::HashMap<std::string::String, std::string::String>,
    #[prost(bool, tag = "16")]
    pub enable_content_negotiation: bool,
    #[prost(message, optional, boxed, tag = "17")]
    pub raw_body: ::std::option::Option<::std::boxed::Box<TypedData>>,
    #[prost(message, repeated, tag = "18")]
    pub identities: ::std::vec::Vec<RpcClaimsIdentity>,
}
pub mod client {
    use super::StreamingMessage;
    use tower_grpc::codegen::client::*;

    /// Interface exported by the server.
    #[derive(Debug, Clone)]
    pub struct FunctionRpc<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> FunctionRpc<T> {
        pub fn new(inner: T) -> Self {
            let inner = grpc::Grpc::new(inner);
            Self { inner }
        }

        /// Poll whether this client is ready to send another request.
        pub fn poll_ready<R>(&mut self) -> futures::Poll<(), grpc::Status>
        where
            T: grpc::GrpcService<R>,
        {
            self.inner.poll_ready()
        }

        /// Get a `Future` of when this client is ready to send another request.
        pub fn ready<R>(self) -> impl futures::Future<Item = Self, Error = grpc::Status>
        where
            T: grpc::GrpcService<R>,
        {
            futures::Future::map(self.inner.ready(), |inner| Self { inner })
        }

        /// Interface exported by the server.
        pub fn event_stream<R, B>(
            &mut self,
            request: grpc::Request<B>,
        ) -> grpc::streaming::ResponseFuture<StreamingMessage, T::Future>
        where
            T: grpc::GrpcService<R>,
            B: futures::Stream<Item = StreamingMessage>,
            B: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static(
                "/AzureFunctionsRpcMessages.FunctionRpc/EventStream",
            );
            self.inner.streaming(request, path)
        }
    }
}
