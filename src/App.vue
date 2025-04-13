<script setup lang="ts">
import TaskCreate from "./components/TaskCreate.vue";
import TasksDisplay from "./components/TasksDisplay.vue";
import UserSettings from "./components/UserSettings.vue";

import { ref } from "vue";

const reloads = ref(0);
</script>

<template>
    <main
        class="h-screen flex flex-col items-center justify-center space-y-4 bg-neutral-900 text-white"
    >
        <h1 class="text-3xl text-green-400">Welcome to Time Sync</h1>
        <UserSettings />
        <TaskCreate @task-creation-success="reloads += 1" />
        <TasksDisplay :key="reloads" />
    </main>
</template>
