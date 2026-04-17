export default function Footer() {
  return (
    <footer className="border-t border-gray-200 dark:border-gray-800 mt-16">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="py-8 text-center text-sm text-gray-600 dark:text-gray-400">
          <p>&copy; {new Date().getFullYear()} Fugue Next.js Example. All rights reserved.</p>
        </div>
      </div>
    </footer>
  );
}
