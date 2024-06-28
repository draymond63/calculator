<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
    // https://icones.js.org/collection/material-symbols-light
    import CalculatorRow from './CalculatorRow.svelte';

	let latexes = [''];
	let results: any = [];

	$: latexes, invoke('evaluate', { input: latexes.join('\n') }).then((res: any) => {
		results = res;
	}).catch((err) => {
		console.error(err);
		results = [];
	});

    function focusRow(index: number) {
        const input = document.getElementById(`input-${index}`);
        const mathquill = input?.querySelector('.mathquill');
        console.log("Input", mathquill);
        (mathquill as HTMLElement)?.focus(); // TODO: This focus does nothing
    }

    function addRow() {
        latexes = [...latexes, ''];
    }

    function dropRow(index: number) {
        if (latexes.length > 1) {
            latexes = latexes.filter((_, i) => i !== index);
        }
    }
</script>

<section>
	{#each latexes as latex, index}
        <CalculatorRow
            {index}
            bind:latex="{latex}"
            result={results[index]}
            on:enter={addRow}
            on:focusUp={() => focusRow(index - 1)}
            on:focusDown={() => focusRow(index + 1)}
            on:drop={() => dropRow(index)}
        />
    {/each}
    <button on:click={addRow}>Add</button>
	<br />
</section>

<style>
	section {
        width: 100%;
        max-width: 400px;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
	}
</style>
