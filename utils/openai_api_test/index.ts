
//LLM-generated (GPT-5.6 Luna)

//curl -X POST http://127.0.0.1:8000/v1/chat/completions -H "Content-Type: application/json" -H "Authorization: Bearer mock-key" -d "{\"model\": \"mock-model\", \"stream\": true, \"messages\": [{\"role\": \"user\", \"content\": \"test\"}]}"
//curl -X POST http://127.0.0.1:8000/v1/chat/completions -H "Content-Type: application/json" -H "Authorization: Bearer mock-key" -d "{\"model\": \"mock-model\", \"stream\": false, \"messages\": [{\"role\": \"user\", \"content\": \"test\"}]}"

const HOST = "127.0.0.1";
const PORT = 8000;

const encoder = new TextEncoder();

const _LOREM_WORDS = [
  "Lorem",
  "ipsum",
  "dolor",
  "sit",
  "amet,",
  "consectetur",
  "adipiscing",
  "elit.",
  "Integer",
  "nec",
  "odio.",
  "Praesent",
  "libero.",
  "Sed",
  "cursus",
  "ante",
  "dapibus",
  "diam.",
];

const LOREM_WORDS = Array.from({ length: 10 }, () => _LOREM_WORDS).flat();

type ChatRequest = {
  model?: string;
  messages?: Array<{
    role: string;
    content: string;
  }>;
  stream?: boolean;
};

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function makeId(): string {
  return `chatcmpl-mock-${crypto.randomUUID()}`;
}

function sse(data: unknown): Uint8Array {
  return encoder.encode(`data: ${JSON.stringify(data)}\n\n`);
}

function doneEvent(): Uint8Array {
  return encoder.encode("data: [DONE]\n\n");
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "access-control-allow-origin": "*",
    },
  });
}

async function* completionStream(
  request: ChatRequest,
): AsyncGenerator<Uint8Array> {
  const id = makeId();
  const model = request.model ?? "mock-model";

  // Send the assistant role first, as OpenAI-compatible APIs usually do.
  yield sse({
    id,
    object: "chat.completion.chunk",
    created: Math.floor(Date.now() / 1000),
    model,
    choices: [
      {
        index: 0,
        delta: {
          role: "assistant",
        },
        finish_reason: null,
      },
    ],
  });

  let counter = 0;
  for (const word of LOREM_WORDS) {
    await sleep(50);
    counter++;
    console.log(counter);
    yield sse({
      id,
      object: "chat.completion.chunk",
      created: Math.floor(Date.now() / 1000),
      model,
      choices: [
        {
          index: 0,
          delta: {
            // Include a trailing space so words do not get concatenated.
            content: `${word} `,
          },
          finish_reason: null,
        },
      ],
    });
  }
  //counter = 0;

  yield sse({
    id,
    object: "chat.completion.chunk",
    created: Math.floor(Date.now() / 1000),
    model,
    choices: [
      {
        index: 0,
        delta: {},
        finish_reason: "stop",
      },
    ],
  });

  yield doneEvent();
}

async function handleChatCompletions(req: Request): Promise<Response> {

  let request: ChatRequest = {};

  try {
    request = await req.json();
  } catch (error) {
    // The request body is not important for this mock server.
    console.error("Invalid JSON:", error);
  }
  console.log(request);

  const model = request.model ?? "mock-model";
  const content = `${LOREM_WORDS.join(" ")} `;
  const id = makeId();
  

  if (request.stream !== true) {
    return jsonResponse({
      id,
      object: "chat.completion",
      created: Math.floor(Date.now() / 1000),
      model,
      choices: [
        {
          index: 0,
          message: {
            role: "assistant",
            content,
          },
          finish_reason: "stop",
        },
      ],
      usage: {
        prompt_tokens: 0,
        completion_tokens: LOREM_WORDS.length,
        total_tokens: LOREM_WORDS.length,
      },
    });
  } else {
    const stream = ReadableStream.from(completionStream(request));

    return new Response(stream, {
      status: 200,
      headers: {
        "content-type": "text/event-stream; charset=utf-8",
        "cache-control": "no-cache",
        "connection": "keep-alive",
        "access-control-allow-origin": "*",
        "x-accel-buffering": "no",
      },
    });
  }
}


function handleModels(): Response {
  return jsonResponse({
    object: "list",
    data: [
      {
        id: "mock-model",
        object: "model",
        created: Math.floor(Date.now() / 1000),
        owned_by: "local-mock",
      },
    ],
  });
}

async function handler(req: Request): Promise<Response> {
  const url = new URL(req.url);

  if (req.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: {
        "access-control-allow-origin": "*",
        "access-control-allow-methods": "POST, GET, OPTIONS",
        "access-control-allow-headers": "content-type, authorization",
      },
    });
  }

  console.log(req);

  if (
    req.method === "POST" &&
    url.pathname === "/v1/chat/completions"
  ) {
    return await handleChatCompletions(req);
  }

  if (req.method === "GET" && url.pathname === "/v1/models") {
    return handleModels();
  }

  if (req.method === "GET" && url.pathname === "/health") {
    return jsonResponse({ status: "ok" });
  }

  return jsonResponse(
    {
      error: {
        message: "Not found",
        type: "invalid_request_error",
      },
    },
    404,
  );
}

console.log(`Mock OpenAI-compatible API: http://${HOST}:${PORT}`);

Deno.serve(
  {
    hostname: HOST,
    port: PORT,
  },
  handler,
);
