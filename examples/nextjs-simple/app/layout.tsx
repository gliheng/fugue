export const metadata = {
  title: 'Next.js on Fugue',
  description: 'A minimal Next.js app running on Fugue FAAS',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
