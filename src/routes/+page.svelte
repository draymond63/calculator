<script lang="ts">
	import { MathQuill } from 'svelte-mathquill';
	// import Graph from './Graph.svelte';
	import { invoke } from '@tauri-apps/api/tauri';

	let latex = '2m/5s';
	let response: any = '';

	$: latex, invoke('evaluate', { input: latex }).then((res) => {
		console.log(res);
		response = res
	})

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
	<p>{latex} = {response}</p>
	<br />
	<!-- <Graph equation="{latex}" /> -->
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
