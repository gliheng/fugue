// Cloudflare Workers compatible handler
export default {
  async fetch(request, env, ctx) {
    // Parse request body
    let data = {};
    try {
      data = await request.json();
    } catch (e) {
      data = {};
    }

    // Call handler function
    const result = handler(data);

    // Return response
    return new Response(JSON.stringify(result), {
      headers: { 'Content-Type': 'application/json' }
    });
  }
};

// Handler function
function handler(event) {
  return {
    message: "Hello " + (event.name || "World"),
    timestamp: Date.now(),
    event: event
  };
}
