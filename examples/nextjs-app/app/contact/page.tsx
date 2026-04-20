'use client';

import { useState } from 'react';
import {
  Card,
  Button,
  Input,
  Label,
  ListBox,
  Select,
  TextArea,
  Form,
  Fieldset,
  FieldGroup,
  TextField,
  FieldError,
  Description
} from '@heroui/react';
import { Send, Check, Mail, MessageCircle, FileText } from 'lucide-react';

export default function Contact() {
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

    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1500));

    setSubmitted(true);

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
          <Form onSubmit={handleSubmit}>
            <Fieldset>
              <FieldGroup>
                <TextField name="name" isRequired>
                  <Label>Name</Label>
                  <Input placeholder="Your name" aria-label="Name" />
                  <FieldError />
                </TextField>

                <TextField name="email" type="email" isRequired>
                  <Label>Email</Label>
                  <Input placeholder="your@email.com" aria-label="Email" />
                  <FieldError />
                </TextField>

                <TextField name="subject" isRequired>
                  <Label>Subject</Label>
                  <Select name="subject" placeholder="Select a subject" aria-label="Subject">
                    <Select.Trigger>
                      <Select.Value />
                      <Select.Indicator />
                    </Select.Trigger>
                    <Select.Popover>
                      <ListBox aria-label="Subject options">
                        {subjects.map((subject) => (
                          <ListBox.Item key={subject} id={subject} textValue={subject}>
                            {subject}
                            <ListBox.ItemIndicator />
                          </ListBox.Item>
                        ))}
                      </ListBox>
                    </Select.Popover>
                  </Select>
                </TextField>

                <TextField name="message" isRequired>
                  <Label>Message</Label>
                  <TextArea
                    placeholder="Tell us more about your project..."
                    rows={6}
                    aria-label="Message"
                  />
                  <FieldError />
                </TextField>
              </FieldGroup>

              <Button
                type="submit"
                className="w-full bg-blue-600 text-white hover:bg-blue-700 mt-6"
                size="lg"
              >
                Send Message
                <Send className="w-5 h-5 ml-2" />
              </Button>

              {submitted && (
                <div className="p-4 bg-green-50 dark:bg-green-900/30 border border-green-200 dark:border-green-800 rounded-lg mt-6">
                  <div className="flex items-center gap-2 text-green-800 dark:text-green-200">
                    <Check className="w-5 h-5" />
                    <span className="font-medium">Message Sent!</span>
                  </div>
                  <p className="text-sm text-green-700 dark:text-green-300 mt-1">
                    Thank you for contacting us. We'll get back to you soon.
                  </p>
                </div>
              )}
            </Fieldset>
          </Form>
        </Card>

        {/* Contact Info */}
        <div className="grid md:grid-cols-3 gap-6">
          <Card className="p-6">
            <div className="text-center">
              <Mail className="w-8 h-8 text-blue-600 mx-auto mb-3" />
              <h3 className="font-semibold mb-2">Email</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">support@fugue.dev</p>
            </div>
          </Card>

          <Card className="p-6">
            <div className="text-center">
              <MessageCircle className="w-8 h-8 text-blue-600 mx-auto mb-3" />
              <h3 className="font-semibold mb-2">Community</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">Join our Discord</p>
            </div>
          </Card>

          <Card className="p-6">
            <div className="text-center">
              <FileText className="w-8 h-8 text-blue-600 mx-auto mb-3" />
              <h3 className="font-semibold mb-2">Documentation</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">Read the docs</p>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
