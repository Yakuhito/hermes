import type { NextConfig } from "next";


const nextConfig = {
  webpack: (config: any) => { 
    config.externals.push('pino-pretty', 'lokijs', 'encoding')
    return config
  }
}

export default nextConfig;
