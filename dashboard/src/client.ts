import { createPromiseClient, Interceptor } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { NodeControl } from "./gen/control_connect";

// Simple interceptor to attach the same control token used by the REST dashboard.
const authInterceptor: Interceptor = (next) => async (req) => {
  try {
    const token = localStorage.getItem('aamn_token');
    if (token) {
      req.header.set("Authorization", `Bearer ${token}`);
    }
  } catch {
    // Ignore storage errors and continue without auth header.
  }
  return next(req);
};

// Configure gRPC-Web transport towards the AAMN node's grpc API
const transport = createGrpcWebTransport({
  baseUrl: "http://localhost:50051",
  interceptors: [authInterceptor],
});

// Create the unified client used by React components
export const client = createPromiseClient(NodeControl, transport);
