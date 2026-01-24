<script lang="ts">
	import { onMount } from 'svelte';

	interface TerminalLine {
		type: 'prompt' | 'input' | 'generating' | 'output' | 'warning' | 'confirm' | 'text' | 'result';
		text: string;
		delay?: number;
	}

	interface Props {
		lines: TerminalLine[];
		title?: string;
	}

	let { lines, title = 'hi-shell' }: Props = $props();

	let visibleLines = $state<TerminalLine[]>([]);
	let currentLineIndex = $state(0);
	let currentCharIndex = $state(0);
	let isTyping = $state(false);
	let showCursor = $state(true);
	let generatingDots = $state('');

	function sleep(ms: number): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	async function animateGenerating(): Promise<void> {
		for (let i = 0; i < 3; i++) {
			generatingDots = '.';
			await sleep(120);
			generatingDots = '..';
			await sleep(120);
			generatingDots = '...';
			await sleep(120);
			generatingDots = '';
		}
	}

	async function runAnimation(): Promise<void> {
		while (true) {
			visibleLines = [];
			currentLineIndex = 0;

			for (const line of lines) {
				if (line.type === 'input') {
					isTyping = true;
					let typedText = '';
					visibleLines = [...visibleLines, { ...line, text: '' }];

					for (let i = 0; i < line.text.length; i++) {
						typedText += line.text[i];
						visibleLines = [
							...visibleLines.slice(0, -1),
							{ ...line, text: typedText }
						];
						await sleep(line.delay || 40);
					}
					isTyping = false;
					await sleep(200);
				} else if (line.type === 'generating') {
					visibleLines = [...visibleLines, line];
					await animateGenerating();
					visibleLines = visibleLines.slice(0, -1);
				} else {
					visibleLines = [...visibleLines, line];
					await sleep(line.delay || 100);
				}
				currentLineIndex++;
			}

			await sleep(3000);
		}
	}

	onMount(() => {
		const cursorInterval = setInterval(() => {
			showCursor = !showCursor;
		}, 530);

		runAnimation();

		return () => clearInterval(cursorInterval);
	});

	function getLineClass(type: string): string {
		switch (type) {
			case 'prompt':
				return 'text-cyan-400';
			case 'output':
				return 'text-emerald-400';
			case 'warning':
				return 'text-yellow-400';
			case 'confirm':
				return 'text-zinc-400';
			case 'text':
				return 'text-zinc-300';
			default:
				return 'text-zinc-100';
		}
	}
</script>

<div class="overflow-hidden rounded-lg border border-zinc-700 bg-zinc-900 shadow-2xl">
	<div class="flex items-center gap-2 border-b border-zinc-700 bg-zinc-800 px-4 py-3">
		<div class="h-3 w-3 rounded-full bg-red-500"></div>
		<div class="h-3 w-3 rounded-full bg-yellow-500"></div>
		<div class="h-3 w-3 rounded-full bg-green-500"></div>
		<span class="ml-2 text-sm text-zinc-400">{title}</span>
	</div>

	<div class="h-64 overflow-y-auto p-4 font-mono text-sm">
		{#each visibleLines as line, i}
			<div class="flex items-start gap-2">
				{#if line.type === 'prompt' || line.type === 'input'}
					<span class="text-green-400">❯</span>
					<span class="text-cyan-400">hi-shell</span>
					<span class="text-zinc-100">{line.text}</span>
					{#if i === visibleLines.length - 1 && isTyping}
						<span class="inline-block w-2 {showCursor ? 'bg-zinc-100' : ''}">&nbsp;</span>
					{/if}
				{:else if line.type === 'generating'}
					<div class="flex items-center gap-2">
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-emerald-400 border-t-transparent"
						></div>
						<span class="text-emerald-400">Generating{generatingDots}</span>
					</div>
				{:else if line.type === 'output'}
					<div class="w-full rounded bg-zinc-800 px-3 py-2">
						<code class="text-emerald-400">{line.text}</code>
					</div>
				{:else if line.type === 'confirm'}
					<span class="text-zinc-400">
						Execute? [<span class="text-emerald-400">y</span>/<span class="text-zinc-300">N</span>]
						{#if i === visibleLines.length - 1}
							<span class="inline-block w-2 {showCursor ? 'bg-zinc-100' : ''}">&nbsp;</span>
						{/if}
					</span>
				{:else if line.type === 'result'}
					<span class="whitespace-pre-wrap text-zinc-300">{line.text}</span>
				{:else}
					<span class={getLineClass(line.type)}>{line.text}</span>
				{/if}
			</div>
		{/each}
	</div>
</div>
