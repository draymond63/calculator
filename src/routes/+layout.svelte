<script>
	import { MathQuillSetup } from 'svelte-mathquill';
	import { SvelteToast } from '@zerodevx/svelte-toast';
	import './styles.css';

	let toastOptions = { reversed: true, intro: { y: 192 } };
</script>

<div class="app">
	<SvelteToast options={toastOptions} />
	<MathQuillSetup />
	<main>
		<slot />
	</main>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 1rem;
		width: 100%;
		max-width: 64rem;
		margin: 0 auto;
		box-sizing: border-box;
	}
</style>
