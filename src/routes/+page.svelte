<script lang="ts">
	import Calculator from "./Calculator.svelte";
	// import Graph from './Graph.svelte';
</script>

<section>
  	<h1>Welcome to<br />Dansmos</h1>
	<Calculator />
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
