'use client';

import { useState } from 'react';
import { Card, Button, Input, Label, ListBox, Select, TextArea } from '@heroui/react';

export default function Contact() {
  const [form, setForm] = useState({
    name: '',
    email: '',
    subject: '',
    message: ''
  });
  const [loading, setLoading] = useState(false);
  const [submitted, setSubmitted] = useState(false);

  const subjects = [
    'General Inquiry',
    'Technical Support',
    'Deployment Help',
    'Feature Request',
    'Partnership'
  ];

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1500));

    setSubmitted(true);
    setForm({ name: '', email: '', subject: '', message: '' });
    setLoading(false);

    setTimeout(() => setSubmitted(false), 5000);
  };

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div className="py-16 max-w-3xl mx-auto">
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold mb-4">Get in Touch</h1>
          <p className="text-xl text-gray-600 dark:text-gray-400">
            Have questions about deploying on Fugue? We'd love to hear from you.
          </p>
        </div>

        <Card className="p-6 mb-12">
          <form onSubmit={handleSubmit} className="space-y-6">
            <div>
              <label className="block text-sm font-medium mb-2">Name *</label>
              <Input
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder="Your name"
                required
                size="lg"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Email *</label>
              <Input
                type="email"
                value={form.email}
                onChange={(e) => setForm({ ...form, email: e.target.value })}
                placeholder="your@email.com"
                required
                size="lg"
              />
            </div>

            <div>
              <Label>Subject *</Label>
              <Select
                selectedKey={form.subject}
                onSelectionChange={(key) => setForm({ ...form, subject: key as string })}
                placeholder="Select a subject"
                required
                size="lg"
              >
                <Select.Trigger>
                  <Select.Value />
                  <Select.Indicator />
                </Select.Trigger>
                <Select.Popover>
                  <ListBox>
                    {subjects.map((subject) => (
                      <ListBox.Item key={subject} id={subject} textValue={subject}>
                        {subject}
                        <ListBox.ItemIndicator />
                      </ListBox.Item>
                    ))}
                  </ListBox>
                </Select.Popover>
              </Select>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Message *</label>
              <TextArea
                value={form.message}
                onChange={(e) => setForm({ ...form, message: e.target.value })}
                placeholder="Tell us more about your project..."
                rows={6}
                required
                size="lg"
              />
            </div>

            <Button
              type="submit"
              isLoading={loading}
              className="w-full bg-blue-600 text-white hover:bg-blue-700"
              size="lg"
            >
              Send Message
              <svg className="w-5 h-5 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
              </svg>
            </Button>

            {submitted && (
              <div className="p-4 bg-green-50 dark:bg-green-900/30 border border-green-200 dark:border-green-800 rounded-lg">
                <div className="flex items-center gap-2 text-green-800 dark:text-green-200">
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                  <span className="font-medium">Message Sent!</span>
                </div>
                <p className="text-sm text-green-700 dark:text-green-300 mt-1">
                  Thank you for contacting us. We'll get back to you soon.
                </p>
              </div>
            )}
          </form>
        </Card>

        {/* Contact Info */}
        <div className="grid md:grid-cols-3 gap-6">
          <Card className="p-6">
            <div className="text-center">
              <svg className="w-8 h-8 text-blue-600 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
              <h3 className="font-semibold mb-2">Email</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">support@fugue.dev</p>
            </div>
          </Card>

          <Card className="p-6">
            <div className="text-center">
              <svg className="w-8 h-8 text-blue-600 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
              <h3 className="font-semibold mb-2">Community</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">Join our Discord</p>
            </div>
          </Card>

          <Card className="p-6">
            <div className="text-center">
              <svg className="w-8 h-8 text-blue-600 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <h3 className="font-semibold mb-2">Documentation</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">Read the docs</p>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
