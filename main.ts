const dir = {
  "/.btn/&/.active": ".btn.active { display: flex; }",
  "/.btn": ".btn { font-size: 20px; }",
};

Deno.serve({ port: 1979 }, async (req) => {
  const file = await Deno.readFile("./dir.html");
  return new Response(file, { status: 200 });
});
