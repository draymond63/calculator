<script lang="ts">
	import { MathQuill } from 'svelte-mathquill';
	import { invoke } from '@tauri-apps/api/tauri';
    import copyicon from '$lib/images/copy-icon.svg';

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

    function focusRow(index: number) {
        const input = document.getElementById(`input-${index}`);
        const mathquill = input?.querySelector('.mathquill');
        console.log("Input", mathquill);
        (mathquill as HTMLElement)?.focus(); // TODO: This focus does nothing
    }

    function addLatex() {
        latexes = [...latexes, ''];
    }

    function dropRow(index: number) {
        if (latexes.length > 1) {
            latexes = latexes.filter((_, i) => i !== index);
        }
    }

    function copyClipboard(line: string) {
        navigator.clipboard.writeText(line);
        console.log("Copied to clipboard", line);
    }

	const autoCommands = 'pi theta phi sqrt sum prod int';
	const autoOperatorNames = 'sin cos tan log exp lim';
</script>

<section>
	{#each latexes as latex, i}
        <div class="input-row" id={`input-${i}`}>
            <MathQuill
                bind:latex="{latex}"
                config={({ autoCommands, autoOperatorNames })}
                on:enter={addLatex}
                on:upOutOf={() => focusRow(i - 1)}
                on:downOutOf={() => focusRow(i + 1)}
                on:deleteOutOf={() => dropRow(i)}
                class="mathquill"
                noBorderOutline
                autofocus
            />
            {#if results[i]}
                <p>{results[i]}</p>
            {/if}
            <button class="icon" on:click={() => copyClipboard(latex)}>
                <img src={copyicon} alt="Copy" />
            </button>
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

    .input-row {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;
        width: 100%;
        border: 1px solid black;
        box-sizing: content-box;
        padding: .2rem;
    	font-size: 1.2em;
    }

    .icon {
        background: none;
        border: none;
        cursor: pointer;        
    }
</style>
