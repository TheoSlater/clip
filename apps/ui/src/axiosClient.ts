import axios from "axios";

export const axiosClient = axios.create({
    baseURL: "http://localhost:43123",
    withCredentials: false,
    timeout: 10000,
});
