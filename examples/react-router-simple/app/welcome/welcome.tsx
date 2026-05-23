export function Welcome({ message }: { message: string }) {
  return (
    <div style={{ maxWidth: 800, margin: "0 auto", padding: "2rem", fontFamily: "system-ui, -apple-system, sans-serif" }}>
      <h1 style={{ color: "#00dc82", marginBottom: "1rem" }}>Welcome to React Router on Fugue!</h1>
      <p>This is a minimal React Router application running on the Fugue FAAS platform.</p>
      <div style={{ background: "#f5f5f5", padding: "1.5rem", borderRadius: 8, marginTop: "2rem" }}>
        <p style={{ margin: "0.5rem 0" }}><strong>Framework:</strong> React Router 7.x</p>
        <p style={{ margin: "0.5rem 0" }}><strong>Runtime:</strong> Cloudflare Workers via workerd</p>
        <p style={{ margin: "0.5rem 0" }}><strong>Server:</strong> Vite + React Router SSR</p>
      </div>
      <div style={{ background: "#f5f5f5", padding: "1.5rem", borderRadius: 8, marginTop: "1rem" }}>
        <p style={{ margin: "0.5rem 0" }}><strong>Message:</strong> {message}</p>
      </div>
    </div>
  );
}