<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
    import { listen } from '@tauri-apps/api/event'
    import { toast } from '@zerodevx/svelte-toast';
    import CalculatorRow from './CalculatorRow.svelte';


	listen('open-file', (event: { event: String, payload: String }) => {
        latexes = event.payload?.split('\n') || [''];
        toast.push("Opened file!");
	})
	listen('save-to-path', (event: { event: String, payload: String }) => {
        // TODO: Check promise from this invoke
        invoke('save_file', { path: event.payload, content: latexes.join('\n') });
        toast.push("Saved successfully!");
	})

    let mode: 'float' | 'complex' | 'units' = 'float';
	let latexes = [''];
	let results: any = [];
    let has_auto_updated_mode = false;

	$: (mode, latexes), invoke(`evaluate_${mode}`, { input: latexes.join('\n') }).then((res: any) => {
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

    function updateMode(new_mode:  'complex' | 'units') {
        if (!has_auto_updated_mode) {
            mode = new_mode;
            toast.push(`Switched to ${new_mode} mode!`);
            has_auto_updated_mode = true;
        }
    }
</script>

<section>
	{#each latexes as latex, index}
        <CalculatorRow
            {index}
            bind:latex="{latex}"
            result={results[index]}
            is_last={index == latexes.length - 1}
            on:detectMode={(e) => updateMode(e.detail.mode)}
            on:enter={addRow}
            on:focusUp={() => focusRow(index - 1)}
            on:focusDown={() => focusRow(index + 1)}
            on:drop={() => dropRow(index)}
        />
    {/each}
    <button on:click={addRow}>Add</button>
</section>

<style>
	section {
        width: 100%;
        max-width: 500px;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
	}

    button {
        margin-top: .5rem;
    }
</style>
