import { createPromiseClient } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { NodeControl } from "./gen/control_connect";

// Configure gRPC-Web transport towards the AAMN node's grpc API
const transport = createGrpcWebTransport({
  baseUrl: "http://localhost:50051",
});

// Create the unified client used by React components
export const client = createPromiseClient(NodeControl, transport);
