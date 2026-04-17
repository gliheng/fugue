import type { Metadata } from 'next';
import Navigation from './components/Navigation';
import Footer from './components/Footer';
import './globals.css';

export const metadata: Metadata = {
  title: 'Fugue + Next.js',
  description: 'Next.js application running on Fugue serverless platform',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <Navigation />
        <main>{children}</main>
        <Footer />
      </body>
    </html>
  );
}
