import axios from 'axios'
import { NetworkError, AuthError, ClientError, ServerError } from './errors'

const baseURL = import.meta.env.VITE_API_BASE_URL || `${window.location.protocol}//${window.location.hostname}:3000`
const authToken = import.meta.env.VITE_AUTH_TOKEN || 'static-token'

export const apiClient = axios.create({ baseURL })

apiClient.interceptors.request.use((config) => {
  config.headers.Authorization = `Bearer ${authToken}`
  return config
})

apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (!axios.isAxiosError(error)) {
      return Promise.reject(error)
    }

    if (!error.response) {
      return Promise.reject(new NetworkError(error))
    }

    const { status, data } = error.response
    const message = typeof data?.message === 'string' ? data.message : error.message

    if (status === 401) {
      console.warn('[auth] Received 401 Unauthorized — token may be invalid or expired')
      return Promise.reject(new AuthError(data))
    }

    if (status >= 400 && status < 500) {
      return Promise.reject(new ClientError(status, message, data))
    }

    if (status >= 500) {
      return Promise.reject(new ServerError(status, message, data))
    }

    return Promise.reject(error)
  },
)
