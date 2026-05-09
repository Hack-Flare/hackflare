import React from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import "./global.css";
import HomePage from "@/pages/HomePage";
import DashboardShell from "@/pages/DashboardShell";
import AdminPage from "@/pages/AdminPage";

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/dash/*" element={<DashboardShell />} />
        <Route path="/admin" element={<AdminPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
