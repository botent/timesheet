export interface Task {
  id: String;
  title: String;
  status: TaskStatus;
  created: Date;
}

export enum TaskStatus {
  RUNNING = "Running",
  COMPLETED = "Completed",
  PAUSED = "Paused",
  NOT_STARTED = "Not yet started",
}
