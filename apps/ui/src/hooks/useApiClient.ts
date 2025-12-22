import { AxiosRequestConfig } from "axios";
import { axiosClient } from "../axiosClient";

export const useApiClient = () => {
    const buildHeaders = (): Record<string, string> => ({
        "Content-Type": "application/json",
        Origin: "http://localhost:43123",
    });

    const get = async <T>(
        url: string,
        config?: AxiosRequestConfig,
    ): Promise<T> => {
        const res = await axiosClient.get<T>(url, {
            ...config,
            headers: {
                ...buildHeaders(),
                ...(config?.headers ?? {}),
            },
        });

        if (res.status === 400 || res.status === 500) {
            throw new Error("Unknown Error");
        }

        return res.data;
    };

    const post = async <T, B = unknown>(
        url: string,
        body: B,
        config?: AxiosRequestConfig,
    ): Promise<T> => {
        const res = await axiosClient.post<T>(url, body, {
            ...config,
            headers: {
                ...buildHeaders(),
                ...(config?.headers ?? {}),
            },
        });

        if (res.status === 400 || res.status === 500) {
            throw new Error("Unknown Error");
        }

        return res.data;
    };

    const put = async <T, B = unknown>(
        url: string,
        body: B,
        config?: AxiosRequestConfig,
    ): Promise<T> => {
        const res = await axiosClient.put<T>(url, body, {
            ...config,
            headers: {
                ...buildHeaders(),
                ...(config?.headers ?? {}),
            },
        });

        if (res.status === 400 || res.status === 500) {
            throw new Error("Unknown Error");
        }

        return res.data;
    };

    const del = async <T>(
        url: string,
        config?: AxiosRequestConfig,
    ): Promise<T> => {
        const res = await axiosClient.delete<T>(url, {
            ...config,
            headers: {
                ...buildHeaders(),
                ...(config?.headers ?? {}),
            },
        });

        if (res.status === 400 || res.status === 500) {
            throw new Error("Unknown Error");
        }

        return res.data;
    };

    return {
        get,
        post,
        put,
        del,
    };
};
