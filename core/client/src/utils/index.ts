import { AxiosError } from "axios";

export function formatAxiosError(error: AxiosError): string {
  const status = error.response?.status || "N/A";
  const statusText = error.response?.statusText || "N/A";
  const url = error.config?.url || "N/A";
  const method = error.config?.method || "N/A";

  return `Axios Error:
  - Message: ${error.message}
  - Status: ${status} ${statusText}
  - Method: ${method.toUpperCase()}
  - URL: ${url}`;
}
