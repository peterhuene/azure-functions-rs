// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_FUNCTION_RPC_EVENT_STREAM: ::grpcio::Method<super::FunctionRpc::StreamingMessage, super::FunctionRpc::StreamingMessage> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Duplex,
    name: "/AzureFunctionsRpcMessages.FunctionRpc/EventStream",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct FunctionRpcClient {
    client: ::grpcio::Client,
}

impl FunctionRpcClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        FunctionRpcClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn event_stream_opt(&self, opt: ::grpcio::CallOption) -> ::grpcio::Result<(::grpcio::ClientDuplexSender<super::FunctionRpc::StreamingMessage>, ::grpcio::ClientDuplexReceiver<super::FunctionRpc::StreamingMessage>)> {
        self.client.duplex_streaming(&METHOD_FUNCTION_RPC_EVENT_STREAM, opt)
    }

    pub fn event_stream(&self) -> ::grpcio::Result<(::grpcio::ClientDuplexSender<super::FunctionRpc::StreamingMessage>, ::grpcio::ClientDuplexReceiver<super::FunctionRpc::StreamingMessage>)> {
        self.event_stream_opt(::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait FunctionRpc {
    fn event_stream(&self, ctx: ::grpcio::RpcContext, stream: ::grpcio::RequestStream<super::FunctionRpc::StreamingMessage>, sink: ::grpcio::DuplexSink<super::FunctionRpc::StreamingMessage>);
}

pub fn create_function_rpc<S: FunctionRpc + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_duplex_streaming_handler(&METHOD_FUNCTION_RPC_EVENT_STREAM, move |ctx, req, resp| {
        instance.event_stream(ctx, req, resp)
    });
    builder.build()
}
