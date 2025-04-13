<script setup lang="ts">
import { ref } from "vue";
import { TaskStatus, type Task } from "../types";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits(["taskCreationSuccess"]);

const task = ref<Task>({
    title: "",
    created: new Date(),
    status: TaskStatus.NOT_STARTED,
});

async function createTask() {
    // invoke command to create and save task
    console.log(task.value);
    await invoke("create_task", { taskData: task.value })
        .then(() => emit("taskCreationSuccess"))
        .catch((e) => console.log(e));
}
</script>

<template>
    <section class="w-full flex flex-col items-center justify-start">
        <form
            @submit.prevent="createTask"
            class="p-4 flex flex-col items-start justify-start space-y-4"
        >
            <input
                type="text"
                required
                placeholder="Title of the task"
                class="rounded-md bg-neutral-800 focus:ring-1 focus:ring-green-400"
                v-model="task.title"
            />
            <select v-model="task.status">
                <option v-for="opt in TaskStatus" key="opt">{{ opt }}</option>
            </select>

            <button type="submit">Submit</button>
        </form>
    </section>
</template>
