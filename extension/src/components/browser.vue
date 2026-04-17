<template>
	<div class="flex justify-between items-center">
		<h1 class="text-lg font-bold mb-4">Browser Configurations</h1>
		<Button icon="pi pi-refresh" size="small" severity="danger" @click="reset()" />
	</div>
	<div class="mb-4">
		<!-- <div v-for="i in browser">{{ i }}</div> -->
		<OrderList v-model="browser" dataKey="id" breakpoint="575px" scrollHeight="20rem">
			<template #option="{ option, selected, index }">
				<div class="flex flex-wrap p-1 items-center gap-4 w-full">
					<!-- <img class="w-12 shrink-0 rounded" :src="'https://primefaces.org/cdn/primevue/images/product/' + option.image"
						:alt="option.name" /> -->
					<div class="flex-1 flex flex-col">
						<span class="font-medium text-sm" :class="{ 'text-amber-600': option.is }">{{ option.label }}</span>
						<span
							:class="['text-sm', { 'text-surface-500 dark:text-surface-400': !selected, 'text-inherit': selected }]">{{
								option.path }}</span>
					</div>
					<div @click.stop class="flex gap-x-2">
						<Button icon="pi pi-pencil" class="p-button-sm" @click="edit(index)" :disabled="option.is" />
						<Button icon="pi pi-trash" class="p-button-sm p-button-danger" @click="del(index)" :disabled="option.is" />
					</div>
					<div class="flex items-center h-[-webkit-fill-available]" @click.stop>
						<Checkbox :model-value="option.show ?? true" @update:model-value="(val) => option.show = val" binary
							:input-id="'browser' + index" />
						<!-- <label :for="'browser' + index" class="select-none ml-0.5">Hidden</label> -->
					</div>
				</div>
			</template>
		</OrderList>
		<!-- Browser configuration management will go here -->
	</div>

	<div class="flex justify-between">
		<Button label="Add New Browser" icon="pi pi-plus" class="p-button-sm" @click="add" />
		<!-- <Button label="Save" icon="pi pi-check" class="p-button-primary" /> -->
	</div>
	<Add v-model:visible="visible" v-model:browser="focus"></Add>
	<Toast />
	<ConfirmDialog />
</template>
<script setup lang="ts">
	import { type BrowserConfig, DEFAULT_BROWSER } from '@/provider';
	import { nextTick, ref } from 'vue';
	import Add from './addDialog.vue';
	import { useConfirm } from "primevue/useconfirm";
	import { useToast } from "primevue/usetoast";
	const confirm = useConfirm();
	const toast = useToast();

	const browser = defineModel<BrowserConfig[]>({ default: DEFAULT_BROWSER });

	const visible = ref(false);
	const DEFAULT_BROWSER_CONFIG = { label: '', path: '', id: -1 };
	const focus = ref<BrowserConfig>(DEFAULT_BROWSER_CONFIG);
	const reset = () => {
		confirm.require({
			message: 'Are you sure you want to reset?',
			header: 'Confirmation',
			icon: 'pi pi-exclamation-triangle',
			rejectProps: {
				label: 'Cancel',
				severity: 'secondary',
				outlined: true
			},
			acceptProps: {
				label: 'Reset',
				severity: 'danger'
			},
			accept: () => {
				toast.add({ severity: 'info', summary: 'Confirmed', detail: 'You have accepted', life: 3000 });
				browser.value = DEFAULT_BROWSER;
			},
			reject: () => {
				toast.add({ severity: 'error', summary: 'Rejected', detail: 'You have rejected', life: 3000 });
			}
		});
	};
	const add = async () => {
		focus.value = DEFAULT_BROWSER_CONFIG;
		await nextTick();
		visible.value = true;
	};
	const edit = async (index: number) => {
		focus.value = browser.value[index];
		await nextTick();
		visible.value = true;
	}
	const del = (index: number) => {
		confirm.require({
			message: 'Are you sure you want to delete?',
			header: 'Confirmation',
			icon: 'pi pi-exclamation-triangle',
			rejectProps: {
				label: 'Cancel',
				severity: 'secondary',
				outlined: true
			},
			acceptProps: {
				label: 'Delete',
				severity: 'danger'
			},
			accept: () => {
				toast.add({ severity: 'info', summary: 'Confirmed', detail: 'You have accepted', life: 3000 });
				browser.value.splice(index, 1);
			},
			reject: () => {
				toast.add({ severity: 'error', summary: 'Rejected', detail: 'You have rejected', life: 3000 });
			}
		});
	};

</script>
<style scoped>

	/* 容器设置为 flex */
	:deep(.p-orderlist-controls) {
		display: flex;
		flex-direction: column;
		/* 保持垂直排列 */
	}

	/* 重新定义顺序  */
	/* 上移按钮 */
	:deep(.p-orderlist-controls .p-button:nth-child(1)) {
		order: 2;
	}

	/* 置顶按钮 */

	:deep(.p-orderlist-controls .p-button:nth-child(2)) {
		order: 1;
	}

	/* 下移按钮 */
	:deep(.p-orderlist-controls .p-button:nth-child(3)) {
		order: 3;
	}

	/* 置底按钮 */
	:deep(.p-orderlist-controls .p-button:nth-child(4)) {
		order: 4;
	}



</style>