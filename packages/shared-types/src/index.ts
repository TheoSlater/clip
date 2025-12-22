export interface StatusResponse {
  buffering: boolean;
  bufferSeconds: number;
}

export interface ClipRequest {
  durationSeconds?: number;
}
