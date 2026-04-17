import Link from 'next/link';
import { Card } from '@heroui/react';

export default function Home() {
  const currentTime = new Date().toISOString();

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div className="py-16">
        {/* Hero Section */}
        <div className="text-center mb-16">
          <div className="inline-block px-3 py-1 mb-4 text-sm font-medium text-blue-600 bg-blue-50 dark:bg-blue-900/30 dark:text-blue-400 rounded-full">
            Next.js 16 + HeroUI v3
          </div>
          <h1 className="text-5xl font-bold mb-6">
            Deploy Next.js on the <span className="text-blue-600">Fugue</span> Platform
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto mb-8">
            A modern full-stack application showcasing Next.js 16 with HeroUI v3 running on the Fugue serverless platform.
          </p>
          <div className="flex gap-4 justify-center">
            <Link
              href="/features"
              className="inline-flex items-center gap-2 px-6 py-3 text-base font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
            >
              Explore Features
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
            </Link>
            <Link
              href="/about"
              className="inline-flex items-center gap-2 px-6 py-3 text-base font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            >
              Learn More
            </Link>
          </div>
        </div>

        {/* Features Grid */}
        <div className="grid md:grid-cols-3 gap-6 mb-16">
          <Card className="p-6">
            <div className="flex items-center gap-3 mb-4">
              <svg className="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              <h3 className="text-lg font-semibold">Server Components</h3>
            </div>
            <p className="text-gray-600 dark:text-gray-400">
              React Server Components for optimal performance and SEO with Next.js 16.
            </p>
          </Card>

          <Card className="p-6">
            <div className="flex items-center gap-3 mb-4">
              <svg className="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              <h3 className="text-lg font-semibold">Fast Cold Starts</h3>
            </div>
            <p className="text-gray-600 dark:text-gray-400">
              Optimized for serverless with minimal startup time and efficient resource usage.
            </p>
          </Card>

          <Card className="p-6">
            <div className="flex items-center gap-3 mb-4">
              <svg className="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
              </svg>
              <h3 className="text-lg font-semibold">API Routes</h3>
            </div>
            <p className="text-gray-600 dark:text-gray-400">
              Built-in API routes with Next.js for full-stack development in one framework.
            </p>
          </Card>
        </div>

        {/* Info Panel */}
        <Card className="p-6">
          <h2 className="text-2xl font-bold mb-6">Current Deployment</h2>
          <div className="grid md:grid-cols-2 gap-6">
            <div>
              <p className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Current Time</p>
              <p className="font-mono text-sm">{currentTime}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Runtime</p>
              <p className="font-mono text-sm">Node.js / Next.js</p>
            </div>
            <div>
              <p className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Framework</p>
              <p className="font-mono text-sm">Next.js 16</p>
            </div>
            <div>
              <p className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Platform</p>
              <p className="font-mono text-sm">Fugue Serverless</p>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
