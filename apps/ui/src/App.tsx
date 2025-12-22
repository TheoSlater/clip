import { ToastProvider } from "@heroui/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Route, Routes } from "react-router";
import "./globals.css";
import { Home } from "./views/Home";

function App() {
    return (
        <QueryClientProvider client={new QueryClient()}>
            <ToastProvider />

            <BrowserRouter>
                <Routes>
                    <Route path="/" element={<Home />} />
                </Routes>
            </BrowserRouter>
        </QueryClientProvider>
    );
}

export default App;
