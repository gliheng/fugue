'use client';

import { useState } from 'react';
import Link from 'next/link';
import { Card, Tabs } from '@heroui/react';

export default function Features() {
  const [selectedTab, setSelectedTab] = useState('runtime');

  const tabs = [
    {
      key: 'runtime',
      label: 'Runtime',
      title: 'Next.js Server Engine',
      description: 'Powered by Next.js 16, providing a modern server engine that works everywhere.',
      features: [
        'Cross-platform deployment support',
        'Automatic code splitting',
        'Built-in caching strategies',
        'Hot module replacement in development'
      ],
      code: `// next.config.ts
export default {
  output: 'standalone',
  experimental: {
    serverActions: true
  }
}`
    },
    {
      key: 'rendering',
      label: 'Rendering',
      title: 'Flexible Rendering Modes',
      description: 'Choose between SSR, SSG, or hybrid rendering based on your needs.',
      features: [
        'Server-side rendering (SSR)',
        'Static site generation (SSG)',
        'Incremental static regeneration',
        'React Server Components'
      ],
      code: `// app/page.tsx
export const dynamic = 'force-dynamic';

export default async function Page() {
  const data = await fetch('...');
  return <div>{data}</div>;
}`
    },
    {
      key: 'routing',
      label: 'Routing',
      title: 'File-Based Routing',
      description: 'Automatic route generation based on your file structure.',
      features: [
        'Zero configuration routing',
        'Dynamic route parameters',
        'Nested routes support',
        'Middleware and guards'
      ],
      code: `// app/blog/[id]/page.tsx
export default function Post({
  params
}: {
  params: { id: string }
}) {
  return <div>Post {params.id}</div>;
}`
    },
    {
      key: 'api',
      label: 'API Routes',
      title: 'Built-in API Routes',
      description: 'Create full-stack applications with server API routes.',
      features: [
        'File-based API routing',
        'TypeScript support',
        'Request validation',
        'Automatic serialization'
      ],
      code: `// app/api/hello/route.ts
export async function GET() {
  return Response.json({
    message: 'Hello from Fugue!',
    timestamp: new Date()
  });
}`
    }
  ];

  const currentTab = tabs.find(t => t.key === selectedTab) || tabs[0];

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div className="py-16">
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold mb-4">Platform Features</h1>
          <p className="text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Discover what makes Fugue + Next.js the perfect combination for modern web applications
          </p>
        </div>

        {/* Feature Tabs */}
        <Tabs
          selectedKey={selectedTab}
          onSelectionChange={(key) => setSelectedTab(key as string)}
          className="mb-12"
        >
          <Tabs.ListContainer>
            <Tabs.List aria-label="Features">
              {tabs.map((tab) => (
                <Tabs.Tab key={tab.key} id={tab.key}>
                  {tab.label}
                  <Tabs.Indicator />
                </Tabs.Tab>
              ))}
            </Tabs.List>
          </Tabs.ListContainer>
          {tabs.map((tab) => (
            <Tabs.Panel key={tab.key} id={tab.key} className="py-8">
              <div className="grid md:grid-cols-2 gap-8">
                <div>
                  <h3 className="text-2xl font-bold mb-4">{tab.title}</h3>
                  <p className="text-gray-600 dark:text-gray-400 mb-6">
                    {tab.description}
                  </p>
                  <ul className="space-y-3">
                    {tab.features.map((feature) => (
                      <li key={feature} className="flex items-start gap-2">
                        <svg className="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                        <span className="text-gray-700 dark:text-gray-300">{feature}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
                  <pre className="text-sm overflow-x-auto"><code>{tab.code}</code></pre>
                </div>
              </div>
            </Tabs.Panel>
          ))}
        </Tabs>

        {/* Stats */}
        <div className="grid md:grid-cols-4 gap-6 mb-12">
          <Card className="p-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 mb-2">&lt;100ms</div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Cold Start Time</div>
            </div>
          </Card>
          <Card className="p-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 mb-2">99.9%</div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Uptime SLA</div>
            </div>
          </Card>
          <Card className="p-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 mb-2">Global</div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Edge Network</div>
            </div>
          </Card>
          <Card className="p-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 mb-2">Auto</div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Scaling</div>
            </div>
          </Card>
        </div>

        <div className="text-center">
          <Link
            href="/contact"
            className="inline-flex items-center gap-2 px-6 py-3 text-base font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
          >
            Get Started
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
          </Link>
        </div>
      </div>
    </div>
  );
}
