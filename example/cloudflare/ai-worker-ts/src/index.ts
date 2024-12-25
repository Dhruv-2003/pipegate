/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Bind resources to your worker in `wrangler.toml`. After adding bindings, a type definition for the
 * `Env` object can be regenerated with `npm run cf-typegen`.
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

export interface Env {
	// If you set another name in wrangler.toml as the value for 'binding',
	// replace "AI" with the variable name you defined.
	AI: Ai;
}

import init, { verify_channel_no_state, PaymentChannelVerifier } from 'pipegate';

export default {
	async fetch(request, env, ctx): Promise<Response> {
		const headerInfo = await getHeaders(request);
		console.log(headerInfo);

		// Check for request Auth here

		try {
			await init();

			const rpc_url = 'https://base-sepolia-rpc.publicnode.com';
			const paymentAmount = BigInt(1000);
			const verifier = new PaymentChannelVerifier(rpc_url);

			const updatedChannel = await verifier.verify_request(
				headerInfo.message,
				headerInfo.signature,
				headerInfo.paymentData,
				paymentAmount,
				headerInfo.bodyBytes
			);

			const response = await env.AI.run('@cf/meta/llama-3.1-8b-instruct', {
				prompt: 'What is the origin of the phrase Hello, World',
			});

			return new Response(JSON.stringify(response));
		} catch (error) {
			console.log(error);
			return new Response('Internal or Authentication', { status: 500 });
		}
	},
} satisfies ExportedHandler<Env>;

interface RequiredHeaders {
	timestamp: string;
	signature: string;
	message: string;
	paymentData: string;
	bodyBytes: Uint8Array;
}

async function getHeaders(request: Request): Promise<RequiredHeaders> {
	// const headers= new Headers()
	const timestamp = request.headers.get('X-Timestamp');
	const signature = request.headers.get('X-Signature');
	const message = request.headers.get('X-Message');
	const paymentData = request.headers.get('X-Payment'); // JSON

	if (!timestamp || !signature || !message || !paymentData) {
		throw new Error('Missing required headers');
	}

	// const paymentChannel = JSON.parse(paymentData);
	const bodyBytes = request.body ? new Uint8Array(await request.arrayBuffer()) : new Uint8Array(0);

	return { timestamp, signature, message, paymentData, bodyBytes };
}
