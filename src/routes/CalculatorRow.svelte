<script lang="ts">
    import { MathQuill } from 'svelte-mathquill';
    // https://icones.js.org/collection/material-symbols-light
    import copyicon from '$lib/images/copy-icon.svg';
    import erroricon from '$lib/images/error-icon.svg';
    import { createEventDispatcher } from 'svelte';
    import { toast } from '@zerodevx/svelte-toast';

    export let latex = '';
    export let index = 0;
    export let result = '';
    export let is_last = false;
	const fire = createEventDispatcher();
    
    let parsed_result = {Ok: '', Err: ''};
    let show_error = false;
    let show_copy = false;
    let is_first = index == 0;

    $: result, parseResult(result);

    function parseResult(res: any) {
        console.log("Result", res);
        if (!res) {
            parsed_result = {Ok: '', Err: ''};
        } else if ('Ok' in res) {
            parsed_result = {Ok: String(res.Ok).replaceAll(/\^([0-9]+)/g, "<sup>$1</sup>"), Err: ''};
        } else if ('Err' in res) {
            parsed_result = {Ok: '', Err: parseError(res.Err)};
        } else {
            parsed_result = {Ok: '', Err: JSON.stringify(res)};
        }
    }

    function parseError(err: any) {
        console.warn(JSON.stringify(err));
        if ('ParseError' in err) return err.ParseError.message + `: '${err.ParseError.span.fragment}'`;
        else if ('EvalError' in err) return err.EvalError;
        else if ('UnitError' in err) return err.UnitError;
        else if ('DefinitionNotFoundError' in err) {
            detectMode(err.DefinitionNotFoundError);
            return `Definition not found: '${err.DefinitionNotFoundError}'`;
        }
        else return JSON.stringify(err);
    }

    function detectMode(missing_def: string) {
        if (missing_def == 'i') {
            fire('detectMode', {mode: 'complex'});
        } else if ([
            'm', 'kg', 's', 'A', 'K', 'mol', 'cd', 'rad', 'sr', 'Hz', 'N',
            'Pa', 'J', 'W', 'C', 'V', 'F', 'Ω', 'S', 'Wb', 'T', 'H', 'lm',
            'lx', 'Bq', 'Gy', 'Sv', 'kat', 'ft', 'in', 'yd',
        ].includes(missing_def)) {
            fire('detectMode', {mode: 'units'});
        }
    }

    function copyClipboard(line: string) {
        navigator.clipboard.writeText(line);
        toast.push("Copied to clipboard!")
    }

	const autoCommands = 'pi theta phi sqrt sum prod int';
	const autoOperatorNames = 'sin cos tan log exp lim';
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="input-row" on:mouseenter={() => show_copy = true} on:mouseleave={() => show_copy = false}>
    <div
        class="input-row input-box"
        class:no-bottom-border={!is_last}
        class:rounded-top={is_first}
        class:rounded-bottom={is_last}
        id={`input-${index}`}
    >
        <MathQuill
            bind:latex="{latex}"
            config={({ autoCommands, autoOperatorNames })}
            on:enter={() => fire('enter')}
            on:upOutOf={() => fire('focusUp')}
            on:downOutOf={() => fire('focusDown')}
            on:deleteOutOf={() => fire('drop')}
            class="mathquill"
            noBorderOutline
            autofocus
        />
        {#if parsed_result.Ok}
            <i>{@html parsed_result.Ok}</i>
        {:else if parsed_result.Err}
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div class="error-container" on:mouseenter={() => show_error = true} on:mouseleave={() => show_error = false}>
                <img src={erroricon} class="icon" width="20px" alt="Error" />
                {#if show_error}
                    <div class="modal">{parsed_result.Err}</div>
                {/if}
            </div>
        {/if}
    </div>
    <button class="icon-button" on:click={() => copyClipboard(latex)} class:hide={!show_copy}>
        <img src={copyicon} class="icon" alt="Copy" />
    </button>
</div>

<style>
    .input-row {
        --icon-dim: 30px;
        --border-radius: 8px;
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;
        width: 100%;
        padding-left: var(--icon-dim);
    }
    .input-box {
        border: 1px solid black;
        min-height: var(--icon-dim);
        padding: .2rem;
    	font-size: 1.2em;
    }
    .rounded-top {
        border-top-left-radius: var(--border-radius);
        border-top-right-radius: var(--border-radius);
    }
    .rounded-bottom {
        border-bottom-left-radius: var(--border-radius);
        border-bottom-right-radius: var(--border-radius);
    }

    .no-bottom-border {
        border-bottom: none;
    }

    .icon-button {
        background: none;
        border: none;
        padding: 0;
        height: var(--icon-dim);
    }

    i {
        width: max-content;
        color: grey;
        font-family: 'Times New Roman', Times, serif;
    }

    .icon {
        width: var(--icon-dim);
        padding: 0;
        cursor: pointer;
    }

    .error-container {
        position: relative;
        height: var(--icon-dim);
    }

    .hide {
        opacity: 0;
    }

    .modal {
        position: absolute;
        top: -2rem;
        right: 0;
        background-color: white;
        border: 1px solid black;
        border-radius: .5rem;
        padding: .5rem;
        width: max-content;
        max-width: 500px;
        z-index: 1;
    }
</style>
