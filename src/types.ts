export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  created: Date;
}

export enum TaskStatus {
  RUNNING = "Running",
  COMPLETED = "Completed",
  PAUSED = "Paused",
  NOT_STARTED = "Not yet started",
}
