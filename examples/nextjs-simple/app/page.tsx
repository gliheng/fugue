export default function Home() {
  return (
    <div style={{
      maxWidth: '800px',
      margin: '0 auto',
      padding: '2rem',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      <h1 style={{ color: '#0070f3', marginBottom: '1rem' }}>
        Welcome to Next.js on Fugue!
      </h1>
      <p>This is a minimal Next.js application running on the Fugue FAAS platform.</p>
      <div style={{
        background: '#f5f5f5',
        padding: '1.5rem',
        borderRadius: '8px',
        marginTop: '2rem'
      }}>
        <p style={{ margin: '0.5rem 0' }}><strong>Framework:</strong> Next.js 15.x</p>
        <p style={{ margin: '0.5rem 0' }}><strong>Runtime:</strong> Node.js</p>
        <p style={{ margin: '0.5rem 0' }}><strong>React:</strong> 19.x</p>
      </div>
    </div>
  )
}
