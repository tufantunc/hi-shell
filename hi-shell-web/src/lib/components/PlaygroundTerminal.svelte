<script lang="ts">
	import { onMount } from 'svelte';
	import { findResponse } from '$lib/playground-responses';

	interface Line {
		type: 'prompt' | 'input' | 'generating' | 'output' | 'warning' | 'confirm' | 'result' | 'info';
		text: string;
	}

	let lines = $state<Line[]>([]);
	let inputValue = $state('');
	let isGenerating = $state(false);
	let awaitingConfirm = $state(false);
	let pendingCommand = $state('');
	let terminalEl: HTMLDivElement | undefined = $state();
	let inputEl: HTMLInputElement | undefined = $state();

	onMount(() => {
		lines = [
			{ type: 'info', text: 'Welcome to hi-shell playground! This is a simulated demo.' },
			{ type: 'info', text: 'Type a natural language request and press Enter.' },
			{ type: 'info', text: 'Try: "list files", "docker containers", "git status", "find large files"' },
			{ type: 'info', text: '\u2014'.repeat(30) }
		];
		inputEl?.focus();
	});

	function sleep(ms: number): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	async function handleSubmit() {
		const input = inputValue.trim();
		if (!input || isGenerating) return;

		inputValue = '';
		lines = [...lines, { type: 'input', text: input }];

		isGenerating = true;
		lines = [...lines, { type: 'generating', text: '' }];

		await sleep(800 + Math.random() * 700);

		const response = findResponse(input);
		if (!response) return;

		lines = lines.slice(0, -1);

		if (response.dangerous) {
			lines = [...lines, { type: 'warning', text: '\u26A0 Warning: This command is potentially dangerous' }];
		}

		lines = [...lines, { type: 'output', text: response.command }];

		if (response.dangerous) {
			lines = [...lines, { type: 'confirm', text: '' }];
			awaitingConfirm = true;
			pendingCommand = response.command;
		} else {
			await sleep(400);
			if (response.output) {
				lines = [...lines, { type: 'result', text: response.output }];
			}
		}

		isGenerating = false;
		scrollToBottom();
	}

	function handleConfirm(yes: boolean) {
		awaitingConfirm = false;
		if (yes) {
			lines = [...lines, { type: 'info', text: 'Executing...' }];
			lines = [...lines, { type: 'result', text: 'Command executed (simulated).' }];
		} else {
			lines = [...lines, { type: 'info', text: 'Command cancelled.' }];
		}
		scrollToBottom();
		inputEl?.focus();
	}

	function scrollToBottom() {
		requestAnimationFrame(() => {
			if (terminalEl) {
				terminalEl.scrollTop = terminalEl.scrollHeight;
			}
		});
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !awaitingConfirm) {
			handleSubmit();
		}
	}
</script>

<div class="overflow-hidden rounded-lg border border-zinc-700 bg-zinc-900 shadow-2xl">
	<div class="flex items-center gap-2 border-b border-zinc-700 bg-zinc-800 px-4 py-3">
		<div class="h-3 w-3 rounded-full bg-red-500"></div>
		<div class="h-3 w-3 rounded-full bg-yellow-500"></div>
		<div class="h-3 w-3 rounded-full bg-green-500"></div>
		<span class="ml-2 text-sm text-zinc-400">hi-shell playground</span>
		<span class="ml-auto text-xs text-zinc-500">demo mode</span>
	</div>

	<div class="h-[420px] overflow-y-auto p-4 font-mono text-sm" bind:this={terminalEl}>
		{#each lines as line}
			<div class="flex items-start gap-2 mb-1">
				{#if line.type === 'prompt' || line.type === 'input'}
					<span class="text-green-400">❯</span>
					<span class="text-cyan-400">hi-shell</span>
					<span class="text-zinc-100">{line.text}</span>
				{:else if line.type === 'generating'}
					<div class="flex items-center gap-2">
						<div class="h-4 w-4 animate-spin rounded-full border-2 border-emerald-400 border-t-transparent"></div>
						<span class="text-emerald-400">Generating...</span>
					</div>
				{:else if line.type === 'output'}
					<div class="w-full rounded bg-zinc-800 px-3 py-2">
						<code class="text-emerald-400">{line.text}</code>
					</div>
				{:else if line.type === 'warning'}
					<span class="text-yellow-400">{line.text}</span>
				{:else if line.type === 'confirm'}
					<div class="flex items-center gap-3">
						<span class="text-zinc-400">
							Execute?
						</span>
						<button class="rounded bg-emerald-600 px-2 py-0.5 text-xs text-white hover:bg-emerald-500" onclick={() => handleConfirm(true)}>Yes (y)</button>
						<button class="rounded bg-zinc-600 px-2 py-0.5 text-xs text-white hover:bg-zinc-500" onclick={() => handleConfirm(false)}>No (N)</button>
					</div>
				{:else if line.type === 'result'}
					<span class="whitespace-pre-wrap text-zinc-300">{line.text}</span>
				{:else if line.type === 'info'}
					<span class="text-zinc-500">{line.text}</span>
				{/if}
			</div>
		{/each}

		{#if !isGenerating && !awaitingConfirm}
			<div class="flex items-center gap-2">
				<span class="text-green-400">❯</span>
				<span class="text-cyan-400">hi-shell</span>
				<input
					type="text"
					bind:this={inputEl}
					bind:value={inputValue}
					onkeydown={handleKeydown}
					placeholder="Type a natural language command..."
					class="flex-1 bg-transparent text-zinc-100 placeholder-zinc-600 outline-none"
					autocomplete="off"
				/>
			</div>
		{/if}
	</div>
</div>
