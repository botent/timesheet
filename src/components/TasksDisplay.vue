<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

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
    <section>
        <div v-for="(val, key) in allTasks" :key="key">
            {{ val }}
        </div>
    </section>
</template>
