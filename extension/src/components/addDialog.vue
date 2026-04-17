<template>
	<Dialog v-model:visible="visible" modal :header="type ? 'Add' : 'Edit'" :style="{ width: '80dvw' }">
		<span class="text-surface-500 dark:text-surface-400 block mb-8">
			{{ type ? '' : 'Update your information.' }}</span>
		<div class="flex items-center gap-4 mb-4">
			<label for="name" class="font-semibold w-24">Name</label>
			<InputText id="name" class="flex-auto" autocomplete="off" v-model="browser.label" :invalid="!browser.label"
				spellcheck="false" />
		</div>
		<div class="flex items-center gap-4 mb-2">
			<label for="path" class="font-semibold w-24">Path</label>
			<InputText id="path" class="flex-auto" autocomplete="off" v-model="browser.path" :invalid="!browser.path"
				spellcheck="false" />
		</div>
		<div class="flex items-center gap-4 mb-2">
			<label for="args" class="font-semibold w-24">Args</label>
			<InputText id="args" class="flex-auto" autocomplete="off" v-model="browser.args" spellcheck="false" />
		</div>
		<template #footer>
			<Button label="Cancel" text severity="danger" @click="visible = false" autofocus />
			<Button label="Save" severity="success" @click="save()" autofocus />
		</template>
	</Dialog>
	<Toast />
</template>

<script setup lang="ts">
	import type { BrowserConfig } from '@/provider';
	import { config } from '@/provider';
	import { useToast } from "primevue/usetoast";
	const toast = useToast();

	const visible = defineModel("visible", { default: false, required: true });
	const browser = defineModel<BrowserConfig>("browser", { required: true });
	const type = browser.value.id === -1;

	const save = () => {
		if (!browser.value.label || !browser.value.path) return toast.add({ severity: 'error', summary: 'Error', detail: 'Check your input', life: 3000 });;
		const type = browser.value.id === -1 ? "add" : "edit";
		switch (type) {
			case "add":
				browser.value.id = Date.now();
				config.value.browser.push(browser.value);
				toast.add({ severity: 'success', summary: 'Success', detail: 'Success add', life: 3000 });
				break;
			case "edit":
				const index = config.value.browser.findIndex((b) => b.id === browser.value.id)
				config.value.browser[index] = browser.value;
				toast.add({ severity: 'success', summary: 'Success', detail: 'Success edit', life: 3000 });
				break;
		}
		visible.value = false;
	};
</script>
