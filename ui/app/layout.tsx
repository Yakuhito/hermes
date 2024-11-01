// "use client";

import "./globals.css";
import type { Metadata } from "next";
import { headers } from "next/headers";
import { Inter } from "next/font/google";
import { config } from "@/config";
import Web3ModalProvider from "@/context";
import { Suspense } from "react";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "AppKit Example App",
  description: "AppKit by reown"
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <Web3ModalProvider><Suspense>{children}</Suspense></Web3ModalProvider>
      </body>
    </html>
  );
}