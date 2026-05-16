export async function loader() {
  return new Response("ok", { status: 200 })
}

export async function action({ request }: { request: Request }) {
  const body = await request.json()

  const res = await fetch("/api/v1/slack/contact",
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    }
  )

  return new Response(null, { status: res.ok ? 200 : 500 })
}