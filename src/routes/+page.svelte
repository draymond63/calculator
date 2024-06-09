<script>
	import { MathQuill } from 'svelte-mathquill';
	import Graph from './Graph.svelte';

	let latex = '';
	const autoCommands = 'pi theta phi sqrt sum prod int';
	const autoOperatorNames = 'sin cos tan log exp lim';
</script>

<section>
  	<h1>
		Welcome to<br />Dansmos
	</h1>
	
	<MathQuill 
		bind:latex="{latex}" 
		config={({ autoCommands, autoOperatorNames })}
	/>
	<br />
	<Graph equation="{latex}" />
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 0.6;
	}

	h1 {
		width: 100%;
	}
</style>
