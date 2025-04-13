<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import { CheckIcon, Delete } from "lucide-vue-next";

const allTasks = ref({});
async function fetchAndDisplayTasks() {
    try {
        allTasks.value = await invoke("display_tasks");
        console.log("Tasks:", allTasks.value);
        // Here, you would update your UI with the 'tasks' data
    } catch (error) {
        console.error("Failed to fetch tasks:", error);
        // Display an error message to the user
    }
}

fetchAndDisplayTasks();
</script>

<template>
    <section
        class="w-full min-w-[60%] mx-auto px-6 py-2 rounded-md bg-neutral-800 border border-neutral-700"
    >
        <div
            v-for="(val, key) in allTasks"
            :key="key"
            class="border-b border-neutral-700 py-4"
        >
            <p class="text-xs font-semibold text-start mb-2">
                {{ key }}
            </p>
            <div
                v-for="(valChild, keyChild) in val"
                :key="keyChild"
                class="w-full flex flex-row items-center justify-between space-x-4 mb-2"
            >
                <p class="font-semibold w-1/2">{{ valChild["title"] }}</p>
                <p
                    class="text-xs rounded-md p-1 bg-teal-500/10 border border-teal-600/10"
                >
                    {{ valChild["status"] }}
                </p>
                <div
                    class="place-self-end flex flex-row items-center justify-end space-x-2"
                >
                    <button class="rounded-sm !p-1 border border-neutral-700">
                        <CheckIcon class="size-3" />
                    </button>
                    <button class="rounded-sm !p-1 border border-neutral-700">
                        <Delete class="size-3" />
                    </button>
                </div>
            </div>
        </div>
    </section>
</template>
