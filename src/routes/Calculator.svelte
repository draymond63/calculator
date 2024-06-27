<script lang="ts">
	import { MathQuill } from 'svelte-mathquill';
	import { invoke } from '@tauri-apps/api/tauri';

	let latexes = [''];
	let results: any = [];

	$: latexes, invoke('evaluate', { input: latexes.join('\n') }).then((res: any) => {
		console.log("Response", res);
		results = res.map(parseResult);
	}).catch((err) => {
		console.error(err); 
		results = [];
	});

    function parseResult(res: any) {
        if ('Ok' in res) return res['Ok'];
        else if ('Err' in res) return res['Err'];
        else return JSON.stringify(res);
    }

    function onEnter(index: number) {
        if (index === latexes.length - 1) {
            addLatex();
        }
        focusNextRow();
    }

    function focusNextRow() {
        const inputs = document.querySelectorAll('.mq-editable-field');
        const lastInput = inputs[inputs.length - 1];
        lastInput.focus();
    }

    function addLatex() {
        latexes = [...latexes, ''];
    }

	const autoCommands = 'pi theta phi sqrt sum prod int';
	const autoOperatorNames = 'sin cos tan log exp lim';
</script>

<section>
	{#each latexes as latex, i}
        <div class="input-section" id={`input-{i}`}>
            <MathQuill
                noBorderOutline
                bind:latex="{latex}"
                config={({ autoCommands, autoOperatorNames })}
                on:enter={() => onEnter(i)}
            />
            {#if results[i]}
                <p>{results[i]}</p>
            {/if}
        </div>
    {/each}
    <button on:click={addLatex}>Add</button>
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

    .input-section {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;
        width: 100%;
        border: 1px solid black;
        padding: .2rem;
    }

    .input-section::first-child {
        margin: 50rem;
        width: 100%;
    }
</style>
