import Link from 'next/link';
import { Card } from '@heroui/react';

export default function About() {
  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div className="py-16 max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold mb-8">About Fugue + Next.js</h1>

        <div className="space-y-8">
          <Card className="p-6">
            <h2 className="text-2xl font-semibold mb-4">What is Fugue?</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              Fugue is a modern serverless platform designed for deploying full-stack applications with minimal configuration. It provides fast cold starts, automatic scaling, and seamless integration with popular frameworks.
            </p>
            <p className="text-gray-600 dark:text-gray-400">
              Built on top of cutting-edge runtime technologies, Fugue enables developers to focus on building features rather than managing infrastructure.
            </p>
          </Card>

          <Card className="p-6">
            <h2 className="text-2xl font-semibold mb-4">Why Next.js 16?</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              Next.js 16 is the latest version of the popular React framework, offering:
            </p>
            <ul className="space-y-2 text-gray-600 dark:text-gray-400">
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span>React Server Components for better performance</span>
              </li>
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span>App Router with improved routing and layouts</span>
              </li>
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span>Server Actions for seamless data mutations</span>
              </li>
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span>TypeScript support out of the box</span>
              </li>
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span>Built-in API routes for full-stack development</span>
              </li>
            </ul>
          </Card>

          <Card className="p-6">
            <h2 className="text-2xl font-semibold mb-4">Perfect Together</h2>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              The combination of Next.js 16 and Fugue provides an excellent developer experience:
            </p>
            <div className="grid md:grid-cols-2 gap-4">
              <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <h3 className="font-semibold mb-2 flex items-center gap-2">
                  <svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  Fast Deployment
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Deploy your Next.js app in seconds with automatic optimization
                </p>
              </div>
              <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <h3 className="font-semibold mb-2 flex items-center gap-2">
                  <svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                  </svg>
                  Auto Scaling
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Handle traffic spikes without manual intervention
                </p>
              </div>
              <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <h3 className="font-semibold mb-2 flex items-center gap-2">
                  <svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  Global Edge
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Serve content from the edge for minimal latency
                </p>
              </div>
              <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <h3 className="font-semibold mb-2 flex items-center gap-2">
                  <svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  Cost Efficient
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Pay only for what you use with serverless pricing
                </p>
              </div>
            </div>
          </Card>
        </div>

        <div className="mt-12 text-center">
          <Link
            href="/features"
            className="inline-flex items-center gap-2 px-6 py-3 text-base font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
          >
            Explore Features
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
          </Link>
        </div>
      </div>
    </div>
  );
}
