<script lang="ts">
	import Calculator from "./Calculator.svelte";
	// import Graph from './Graph.svelte';

	let use_units = false;
	let use_complex = false;
	let mode: 'float' | 'complex' | 'units' = 'float';

	$: use_units, updateMode('units', use_units);
	$: use_complex, updateMode('complex', use_complex);

	// TODO: Automatically detect mode
	function updateMode(new_mode: 'units' | 'complex' | 'float', state: boolean) {
		if (mode != new_mode && state) {
			mode = new_mode;
			if (new_mode == 'units') {
				use_complex = false;
			} else if (new_mode == 'complex') {
				use_units = false;
			}
		} else if (mode == new_mode && !state) {
			mode = 'float';
			use_units = false;
			use_complex = false;
		}
	}
</script>

<section>
  	<h1>Dansmos</h1>
	<Calculator mode={mode} />
	<ul>
		<p><input type="checkbox" bind:checked={use_units} /> Units</p>
		<p><input type="checkbox" bind:checked={use_complex} /> Complex Numbers</p>
	</ul>
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
		font-family: 'Courier New', Courier, monospace;
	}
</style>
