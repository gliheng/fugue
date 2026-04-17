<template>
  <UContainer>
    <div class="py-16 max-w-3xl mx-auto">
      <div class="text-center mb-12">
        <h1 class="text-4xl font-bold mb-4">Get in Touch</h1>
        <p class="text-xl text-gray-600 dark:text-gray-400">
          Have questions about deploying on Fugue? We'd love to hear from you.
        </p>
      </div>

      <UCard>
        <UForm :state="form" @submit="onSubmit" class="space-y-6">
          <UFormField label="Name" name="name" required>
            <UInput v-model="form.name" placeholder="Your name" size="lg" />
          </UFormField>

          <UFormField label="Email" name="email" required>
            <UInput v-model="form.email" type="email" placeholder="your@email.com" size="lg" />
          </UFormField>

          <UFormField label="Subject" name="subject" required>
            <USelect
              v-model="form.subject"
              :options="subjects"
              placeholder="Select a subject"
              size="lg"
            />
          </UFormField>

          <UFormField label="Message" name="message" required>
            <UTextarea
              v-model="form.message"
              placeholder="Tell us more about your project..."
              :rows="6"
              size="lg"
            />
          </UFormField>

          <div class="flex gap-4">
            <UButton type="submit" size="lg" :loading="loading" block>
              Send Message
              <template #trailing>
                <UIcon name="i-heroicons-paper-airplane" />
              </template>
            </UButton>
          </div>
        </UForm>
      </UCard>

      <!-- Contact Info -->
      <div class="grid md:grid-cols-3 gap-6 mt-12">
        <UCard>
          <div class="text-center">
            <UIcon name="i-heroicons-envelope" class="w-8 h-8 text-primary mx-auto mb-3" />
            <h3 class="font-semibold mb-2">Email</h3>
            <p class="text-sm text-gray-600 dark:text-gray-400">support@fugue.dev</p>
          </div>
        </UCard>

        <UCard>
          <div class="text-center">
            <UIcon name="i-heroicons-chat-bubble-left-right" class="w-8 h-8 text-primary mx-auto mb-3" />
            <h3 class="font-semibold mb-2">Community</h3>
            <p class="text-sm text-gray-600 dark:text-gray-400">Join our Discord</p>
          </div>
        </UCard>

        <UCard>
          <div class="text-center">
            <UIcon name="i-heroicons-document-text" class="w-8 h-8 text-primary mx-auto mb-3" />
            <h3 class="font-semibold mb-2">Documentation</h3>
            <p class="text-sm text-gray-600 dark:text-gray-400">Read the docs</p>
          </div>
        </UCard>
      </div>

      <!-- Success Notification -->
    </div>
  </UContainer>
</template>

<script setup lang="ts">
const toast = useToast()

const form = reactive({
  name: '',
  email: '',
  subject: '',
  message: ''
})

const subjects = [
  'General Inquiry',
  'Technical Support',
  'Deployment Help',
  'Feature Request',
  'Partnership'
]

const loading = ref(false)

const onSubmit = async () => {
  loading.value = true

  // Simulate API call
  await new Promise(resolve => setTimeout(resolve, 1500))

  toast.add({
    title: 'Message Sent!',
    description: 'Thank you for contacting us. We\'ll get back to you soon.',
    icon: 'i-heroicons-check-circle',
    color: 'success'
  })

  // Reset form
  form.name = ''
  form.email = ''
  form.subject = ''
  form.message = ''

  loading.value = false
}
</script>
