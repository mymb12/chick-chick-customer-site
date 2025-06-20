# Januya - AI-Powered Chicken Health Monitoring System 🐔💻

**Januya is an innovative system designed to revolutionize poultry health management. By leveraging Artificial Intelligence and computer vision, Januya provides proactive insights to safeguard your flock, optimize farm productivity, and enable early detection of potential health issues.**

This repository contains the source code for the Januya landing page and its basic backend components.

---

## ✨ Features

*   **Eye-Catching Landing Page:**
    *   Responsive layout for optimal viewing on all devices (desktop, tablet, mobile).
    *   Animated modal pop-up for a better user experience.
*   **Core System Concepts (as presented on the landing page):**
    *   **Real-time Monitoring:** Continuous observation of poultry health indicators.
    *   **AI-Powered Analytics:** Diagnosis and prediction of health issues using AI.
    *   **Early Disease Detection:** Minimizing losses through proactive alerts.
    *   **User-Friendly Interface:** Accessible via web and mobile (conceptually).
*   **Basic Backend (Rust):**
    *   Serves the static HTML, CSS, and image files.
    *   Handles POST request.
    *   Saves demo request data (Timestamp, Name, Email, Phone, Comments) to a local `demo_requests.csv` file.

---

## Preview

Currently, the project is set up to run locally. See the "Getting Started" section below.

---

## 🛠️ Tech Stack

*   **Frontend:**
    *   HTML
    *   CSS
    *   JavaScript 
*   **Backend:**
    *   Rust (Standard Library, `chrono` crate for timestamps, `urlencoding` for form parsing)
*   **Data Storage (for demo requests):**
    *   CSV file (`demo_requests.csv`)

---

## Getting Started

To get a local copy up and running, follow these simple steps.

### Prerequisites

*   **Rust:** Ensure you have Rust installed. You can get it from [rust-lang.org](https://www.rust-lang.org/tools/install).
*   **Git:** For cloning the repository.

### Installation & Running

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/mymb12/chick-chick-customer-site 
    cd chick-chick-customer-site
    ```

2.  **Build and Run the Rust Server:**
    The Rust server will serve the `index.html` and handle API requests for demo submissions.
    ```bash
    cargo build
    cargo run
    ```
    The server will start listening on `http://127.0.0.1:7878` by default.

3.  **Access the Landing Page:**
    Open your web browser and navigate to:
    `http://127.0.0.1:7878`

4.  **(Optional) Second Local Site for "Смотреть демо" Button:**
    The "Смотреть демо" button is configured to link to `http://127.0.0.1:6564`. If you have another local project running on that port, it will open in a new tab (additionally requires another repo cloned: https://github.com/mymb12/chikc-chick).
---

## 🛣️ Potential Future Enhancements

*   Replace placeholder images with actual product/feature visuals.
*   Implement a more robust database (e.g., SQLite, PostgreSQL) instead of CSV for storing demo requests.
*   Develop a full-fledged admin interface to view and manage demo requests.
*   Add user authentication.
*   Integrate with email services for sending notifications.
*   Deploy the static front-end to a free hosting provider (Netlify, GitHub Pages).
*   Deploy the Rust backend (or a serverless function alternative) to handle form submissions in a live environment.
*   Add more sophisticated animations and micro-interactions.
*   Implement internationalization (i18n) for multiple languages.

