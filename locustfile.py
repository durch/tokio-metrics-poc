from locust import HttpLocust, TaskSet, task


class WebsiteTasks(TaskSet):
    @task(2)
    def fast(self):
        self.client.get("/")

    @task(1)
    def slow(self):
        self.client.get("/slow")


class WebsiteUser(HttpLocust):
    task_set = WebsiteTasks
    min_wait = 1000
    max_wait = 5000
