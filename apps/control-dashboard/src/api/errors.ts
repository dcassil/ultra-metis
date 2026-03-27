export class ApiError extends Error {
  status?: number
  responseBody?: unknown
  originalError?: unknown

  constructor(
    message: string,
    status?: number,
    responseBody?: unknown,
    originalError?: unknown,
  ) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.responseBody = responseBody
    this.originalError = originalError
  }
}

export class NetworkError extends ApiError {
  constructor(originalError?: unknown) {
    super('Network error — unable to reach the server', undefined, undefined, originalError)
    this.name = 'NetworkError'
  }
}

export class AuthError extends ApiError {
  constructor(responseBody?: unknown) {
    super('Unauthorized — authentication required or token invalid', 401, responseBody)
    this.name = 'AuthError'
  }
}

export class ClientError extends ApiError {
  constructor(status: number, message: string, responseBody?: unknown) {
    super(message, status, responseBody)
    this.name = 'ClientError'
  }
}

export class ServerError extends ApiError {
  constructor(status: number, message: string, responseBody?: unknown) {
    super(message, status, responseBody)
    this.name = 'ServerError'
  }
}
