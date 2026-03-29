const COPY_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>`;
const CHECK_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>`;

export function enhanceCodeBlocks(container: HTMLElement) {
	container.querySelectorAll('.code-block-wrapper').forEach((wrapper) => {
		const pre = wrapper.querySelector('pre');
		const btn = wrapper.querySelector('.code-copy-btn');
		if (pre) wrapper.parentNode?.insertBefore(pre, wrapper);
		if (btn) btn.remove();
		wrapper.remove();
	});

	const pres = container.querySelectorAll('pre');

	pres.forEach((pre) => {
		if (pre.parentElement?.classList.contains('code-block-wrapper')) return;

		const codeEl = pre.querySelector('code');
		const text = codeEl?.textContent?.trim() || '';
		const lang = codeEl?.className?.replace('language-', '') || '';

		const wrapper = document.createElement('div');
		wrapper.className = 'code-block-wrapper';

		const btn = document.createElement('button');
		btn.className = 'code-copy-btn';
		btn.title = 'Copy to clipboard';
		btn.setAttribute('aria-label', 'Copy to clipboard');
		btn.innerHTML = COPY_ICON;

		if (lang) {
			const label = document.createElement('span');
			label.className = 'code-lang-label';
			label.textContent = lang;
			wrapper.appendChild(label);
		}

		pre.parentNode?.insertBefore(wrapper, pre);
		wrapper.appendChild(pre);
		wrapper.appendChild(btn);

		btn.addEventListener('click', async () => {
			try {
				await navigator.clipboard.writeText(text);
				btn.innerHTML = CHECK_ICON;
				btn.classList.add('copied');
				setTimeout(() => {
					btn.innerHTML = COPY_ICON;
					btn.classList.remove('copied');
				}, 2000);
			} catch {
				const textarea = document.createElement('textarea');
				textarea.value = text;
				textarea.style.position = 'fixed';
				textarea.style.opacity = '0';
				document.body.appendChild(textarea);
				textarea.select();
				document.execCommand('copy');
				document.body.removeChild(textarea);
				btn.innerHTML = CHECK_ICON;
				btn.classList.add('copied');
				setTimeout(() => {
					btn.innerHTML = COPY_ICON;
					btn.classList.remove('copied');
				}, 2000);
			}
		});
	});
}
