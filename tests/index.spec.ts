import { test, expect } from '@playwright/test';

test('All pages render', async ({ page }) => {
    await page.goto('http://localhost:34115/');
    await expect(page.getByText(/Hello,/)).toBeVisible();
    await page.getByTestId('settings').click();
    await expect(page.getByText("Settings")).toBeVisible();
    await page.getByText("Cancel").click();
    await page.getByTestId('view-logs').click();
    await expect(page.getByText("Your Logs")).toBeVisible();
    await page.getByTestId('add-log').click();
    await expect(page.getByText("Find a Game")).toBeVisible();
    await page.getByTestId('game-card').first().click();
    await expect(page.getByText("New Log")).toBeVisible();
})

test('Viewing logs', async ({ page }) => {
    await page.goto('http://localhost:34115/');
    await page.getByTestId('view-logs').click();
    await expect(page.getByTestId('game-card').first()).toBeVisible();
})
